// The Mac's own views for what GPUI can't draw: a video in AVKit's player and
// a PDF in PDFKit's view, each placed over a GPUI layout slot as the web
// runtime's pages are (desktop/web/native/appkit_hooks.m). media_view.rs
// places, hides and closes them. open_gate.rs is the only caller of
// tb_media_open, with the file the gate opened and checked (decide_media),
// so nothing here decides what may be read.
//
// They read only that open file, never its path again: a film through an
// asset whose bytes come from it (TBFileSource), with QuickTime's references
// to other files forbidden; a PDF from its bytes. A PDF's links go back to
// the open gate, never to the system (Codex's review, 12 Nov).
//
// They behave as the Swift app's do: FilePreviewView in the pane (the
// player's floating controls, sound on, paused on its first frame, sized to
// the film's shape so no black bars show, the picture dragging out as the
// file), and ReviewVideo in the review window (looping from the start, with
// controls and sound on the stage, muted with none among them all).
#import <AVFoundation/AVFoundation.h>
#import <AVKit/AVKit.h>
#import <AppKit/AppKit.h>
#import <PDFKit/PDFKit.h>
#include <stdbool.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

// How a view is used: media_view.rs's Kind, in the same order.
enum {
  TBPaneVideo = 0,
  TBStageVideo = 1,
  TBCellVideo = 2,
  TBPaneDocument = 3,
};

// What a view is showing, for the scripted app's dump (media_view.rs's
// NativeState, field for field).
typedef struct {
  double width;
  double height;
  bool playing;
  bool muted;
  bool controls;
  bool looping;
  float volume;
  int pages;
  bool failed;
} TBMediaState;

// Implemented in Rust (media_view.rs): a PDF's link clicked, for the open
// gate, with the PDF's folder for a relative one; and a view's state
// changed in a way the window should draw again for (a film that failed).
extern void tb_media_said_link(const char *url, const char *base);
extern void tb_media_said_changed(void);

// The biggest PDF read into memory to show.
static const long long TBDocumentLimit = 256LL * 1024 * 1024;

// ---------------------------------------------------------------- the file

// The checked file, held open, serving a film's bytes to AVFoundation as it
// asks for them, from a queue of its own. Closes the file when it goes.
@interface TBFileSource : NSObject <AVAssetResourceLoaderDelegate>
@property(nonatomic, readonly) int fd;
@property(nonatomic, readonly) long long length;
@property(nonatomic, readonly, copy) NSString *contentType;
@property(nonatomic, readonly) dispatch_queue_t queue;
@end

@implementation TBFileSource

- (instancetype)initWithDescriptor:(int)fd length:(long long)length path:(NSString *)path {
  if ((self = [super init])) {
    _fd = fd;
    _length = length;
    NSString *ext = path.pathExtension.lowercaseString;
    _contentType = [ext isEqualToString:@"mov"] ? AVFileTypeQuickTimeMovie : [ext isEqualToString:@"m4v"] ? AVFileTypeAppleM4V : AVFileTypeMPEG4;
    _queue = dispatch_queue_create("threadbear.media.file", DISPATCH_QUEUE_SERIAL);
  }
  return self;
}

- (void)dealloc {
  if (_fd >= 0) {
    close(_fd);
  }
}

- (BOOL)resourceLoader:(AVAssetResourceLoader *)loader shouldWaitForLoadingOfRequestedResource:(AVAssetResourceLoadingRequest *)request {
  AVAssetResourceLoadingContentInformationRequest *info = request.contentInformationRequest;
  if (info) {
    info.contentType = self.contentType;
    info.contentLength = self.length;
    info.byteRangeAccessSupported = YES;
  }
  AVAssetResourceLoadingDataRequest *data = request.dataRequest;
  if (data) {
    long long end = data.requestsAllDataToEndOfResource ? self.length : MIN(self.length, data.requestedOffset + (long long)data.requestedLength);
    long long at = data.currentOffset;
    while (at < end) {
      if (request.isCancelled) {
        return YES;
      }
      size_t chunk = (size_t)MIN(1024LL * 1024, end - at);
      NSMutableData *bytes = [NSMutableData dataWithLength:chunk];
      ssize_t got = pread(self.fd, bytes.mutableBytes, chunk, at);
      if (got <= 0) {
        [request finishLoadingWithError:[NSError errorWithDomain:NSPOSIXErrorDomain code:got < 0 ? errno : EIO userInfo:nil]];
        return YES;
      }
      bytes.length = (NSUInteger)got;
      [data respondWithData:bytes];
      at += got;
    }
  }
  [request finishLoading];
  return YES;
}

// The file at `path` is still the one held open: the same file on the same
// disk, the same size and last change, and not a link put in its place.
- (BOOL)isStillAt:(NSString *)path {
  struct stat held, now;
  if (fstat(self.fd, &held) != 0 || lstat(path.fileSystemRepresentation, &now) != 0) {
    return NO;
  }
  return S_ISREG(now.st_mode) && held.st_dev == now.st_dev && held.st_ino == now.st_ino && held.st_size == now.st_size &&
         held.st_mtimespec.tv_sec == now.st_mtimespec.tv_sec && held.st_mtimespec.tv_nsec == now.st_mtimespec.tv_nsec;
}

@end

// An asset whose bytes come from `source`, under a scheme of its own so
// AVFoundation asks the source for them rather than opening a path, and
// with QuickTime's references to other files (and anywhere else) forbidden,
// so a movie can't pull in a file the gate never checked.
static AVURLAsset *tb_asset(TBFileSource *source, NSString *name) {
  NSURLComponents *url = [NSURLComponents new];
  url.scheme = @"threadbear-media";
  url.host = @"file";
  url.path = [@"/" stringByAppendingString:name];
  AVURLAsset *asset = [AVURLAsset URLAssetWithURL:url.URL options:@{AVURLAssetReferenceRestrictionsKey : @(AVAssetReferenceRestrictionForbidAll)}];
  [asset.resourceLoader setDelegate:source queue:source.queue];
  return asset;
}

// ---------------------------------------------------------------- placing

// The frame for a slot in GPUI's coordinates (points from the window's top
// left), in the parent's own, which may not be flipped. A film fitted to its
// shape inside the slot, when there's one to fit.
static void tb_media_layout(NSView *view, NSRect slot, CGSize film) {
  NSRect frame = slot;
  if (film.width > 0 && film.height > 0) {
    frame = NSIntegralRect(AVMakeRectWithAspectRatioInsideRect(film, slot));
  }
  NSView *parent = view.superview;
  if (parent && !parent.isFlipped) {
    frame.origin.y = NSHeight(parent.bounds) - frame.origin.y - frame.size.height;
  }
  frame.size.width = MAX(0.0, frame.size.width);
  frame.size.height = MAX(0.0, frame.size.height);
  if (!NSEqualRects(view.frame, frame)) {
    view.frame = frame;
  }
}

// Waits on a press to see whether it's a drag (FileDrag.isDrag): yes once the
// mouse has moved 4 points, no when it's let go first, with that mouse-up
// left in the queue so the view under it still sees the click.
static BOOL tb_is_drag(NSEvent *event) {
  NSWindow *window = event.window;
  if (!window) {
    return NO;
  }
  NSPoint start = event.locationInWindow;
  while (YES) {
    NSEvent *next = [window nextEventMatchingMask:NSEventMaskLeftMouseUp | NSEventMaskLeftMouseDragged untilDate:[NSDate distantFuture] inMode:NSEventTrackingRunLoopMode dequeue:NO];
    if (!next || next.type == NSEventTypeLeftMouseUp) {
      return NO;
    }
    [window nextEventMatchingMask:NSEventMaskLeftMouseDragged];
    if (hypot(next.locationInWindow.x - start.x, next.locationInWindow.y - start.y) >= 4) {
      return YES;
    }
  }
}

// ---------------------------------------------------------------- the player

static void *TBStatusContext = &TBStatusContext;

@interface TBPlayerView : AVPlayerView <NSDraggingSource>
@property(nonatomic) int kind;
@property(nonatomic) NSRect slot;
@property(nonatomic) CGSize filmSize;
@property(nonatomic, strong) AVPlayerLooper *looper;
@property(nonatomic, strong) TBFileSource *source;
@property(nonatomic, strong) AVURLAsset *asset;
// The real file's path, for dragging it out once it's checked as the same.
@property(nonatomic, copy) NSString *path;
// What's being watched for a failure: the pane's item, or the looper.
@property(nonatomic, strong) NSObject *watched;
@property(nonatomic, copy) NSString *failure;
@property(nonatomic) BOOL passingClick;
@end

@implementation TBPlayerView

// Clicks count while Threadbear isn't the app in front, as in the Swift app.
- (BOOL)acceptsFirstMouse:(NSEvent *)event {
  return YES;
}

// In the review window, never the window's first responder, so the arrow
// keys keep stepping through the shots after a click on its controls, not
// scrubbing (ReviewVideo.Player). In the pane, Space plays and pauses.
- (BOOL)acceptsFirstResponder {
  return self.kind == TBPaneVideo ? [super acceptsFirstResponder] : NO;
}

- (NSView *)hitTest:(NSPoint)point {
  if (self.hidden) {
    return nil;
  }
  // Among them all, a click is the cell's: GPUI's view under this one
  // enlarges it, as the Swift grid's overlay does.
  if (self.kind == TBCellVideo) {
    return nil;
  }
  NSView *inner = [super hitTest:point];
  if (self.kind != TBPaneVideo || !inner || NSApp.currentEvent.type != NSEventTypeLeftMouseDown) {
    return inner;
  }
  // A left press on the picture itself is taken here, to be a drag or a
  // click; the controls are smaller panels over it (the play bar, the
  // volume), left to the player (FilePreviewView.pressTarget).
  NSRect frame = [inner convertRect:inner.bounds toView:self];
  return frame.size.width >= self.bounds.size.width - 1 && frame.size.height >= self.bounds.size.height - 1 ? self : inner;
}

- (void)mouseDown:(NSEvent *)event {
  // The click handed on below came back up the responders: on up to the
  // window's view, rather than down again, which would loop (THREADBEAR-N).
  if (self.passingClick) {
    [super mouseDown:event];
    return;
  }
  if (self.kind == TBPaneVideo && self.path && tb_is_drag(event)) {
    [self dragFile:event];
    return;
  }
  NSPoint point = [self.superview convertPoint:event.locationInWindow fromView:nil];
  NSView *target = [super hitTest:point] ?: self;
  self.passingClick = YES;
  [target mouseDown:event];
  self.passingClick = NO;
}

// The film dragged out as the file itself, to drop on X or the Finder (Mike
// wanted to post a clip without opening the panel first, 29 Oct). Only while
// the path still leads to the very file that's playing, checked against the
// one held open: a link or another file put there since is never handed on.
// Copy only.
- (void)dragFile:(NSEvent *)event {
  if (![self.source isStillAt:self.path]) {
    NSBeep();
    return;
  }
  NSURL *file = [NSURL fileURLWithPath:self.path isDirectory:NO];
  NSDraggingItem *item = [[NSDraggingItem alloc] initWithPasteboardWriter:file];
  NSImage *image = [NSImage imageWithSystemSymbolName:@"film" accessibilityDescription:nil] ?: [NSImage new];
  NSImageSymbolConfiguration *config = [NSImageSymbolConfiguration configurationWithPointSize:36 weight:NSFontWeightRegular];
  image = [image imageWithSymbolConfiguration:config] ?: image;
  NSSize size = image.size;
  NSPoint point = [self convertPoint:event.locationInWindow fromView:nil];
  [item setDraggingFrame:NSMakeRect(point.x - size.width / 2, point.y - size.height / 2, size.width, size.height) contents:image];
  [self beginDraggingSessionWithItems:@[ item ] event:event source:self];
}

- (NSDragOperation)draggingSession:(NSDraggingSession *)session sourceOperationMaskForDraggingContext:(NSDraggingContext)context {
  return NSDragOperationCopy;
}

// The film's shape, once its track says: the pane's player is that shape in
// the middle of its slot. It fills the slot until then.
- (void)loadFilmSize {
  AVURLAsset *asset = self.asset;
  __weak TBPlayerView *weakSelf = self;
  [asset loadValuesAsynchronouslyForKeys:@[ @"tracks" ]
                       completionHandler:^{
                         AVAssetTrack *track = [asset tracksWithMediaType:AVMediaTypeVideo].firstObject;
                         if (!track) {
                           return;
                         }
                         [track loadValuesAsynchronouslyForKeys:@[ @"naturalSize", @"preferredTransform" ]
                                              completionHandler:^{
                                                CGSize shown = CGSizeApplyAffineTransform(track.naturalSize, track.preferredTransform);
                                                dispatch_async(dispatch_get_main_queue(), ^{
                                                  TBPlayerView *me = weakSelf;
                                                  if (!me || shown.width == 0 || shown.height == 0) {
                                                    return;
                                                  }
                                                  me.filmSize = CGSizeMake(fabs(shown.width), fabs(shown.height));
                                                  [me layoutInSlot];
                                                });
                                              }];
                       }];
}

- (void)layoutInSlot {
  tb_media_layout(self, self.slot, self.kind == TBPaneVideo ? self.filmSize : CGSizeZero);
}

// A film AVFoundation can't play says why in its place (media_view.rs reads
// `failure`), rather than leaving a player that never starts.
- (void)watchForFailure:(NSObject *)object {
  self.watched = object;
  [object addObserver:self forKeyPath:@"status" options:NSKeyValueObservingOptionInitial | NSKeyValueObservingOptionNew context:TBStatusContext];
}

- (void)stopWatching {
  if (self.watched) {
    [self.watched removeObserver:self forKeyPath:@"status" context:TBStatusContext];
    self.watched = nil;
  }
}

- (void)observeValueForKeyPath:(NSString *)keyPath ofObject:(id)object change:(NSDictionary *)change context:(void *)context {
  if (context != TBStatusContext) {
    [super observeValueForKeyPath:keyPath ofObject:object change:change context:context];
    return;
  }
  NSError *error = nil;
  if ([object isKindOfClass:[AVPlayerItem class]] && ((AVPlayerItem *)object).status == AVPlayerItemStatusFailed) {
    error = ((AVPlayerItem *)object).error;
  } else if ([object isKindOfClass:[AVPlayerLooper class]] && ((AVPlayerLooper *)object).status == AVPlayerLooperStatusFailed) {
    error = ((AVPlayerLooper *)object).error;
  } else {
    return;
  }
  NSString *why = error.localizedFailureReason ?: error.localizedDescription ?: @"it isn't a film it can read";
  dispatch_async(dispatch_get_main_queue(), ^{
    if (self.failure) {
      return;
    }
    self.failure = why;
    [self.player pause];
    tb_media_said_changed();
  });
}

@end

// ---------------------------------------------------------------- the PDF

// A PDF's links never reach the system from here: PDFKit's own handling
// hands a clicked link to NSWorkspace, which would open a file link to a
// program or a script with no question. Every link and action that leaves
// the document goes to the open gate instead (tb_media_said_link); moving
// within it stays PDFKit's.

// Hands a link to the gate, with the PDF's folder for a relative one.
static void tb_send_link(NSURL *url, NSString *base) {
  if (url.absoluteString.length) {
    tb_media_said_link(url.absoluteString.UTF8String, base.fileSystemRepresentation);
  }
}

// The PDF view's delegate, an object of its own: a PDFView that's its own
// delegate loops forever, as PDFView passes some of its own calls on to it.
@interface TBDocumentLinks : NSObject <PDFViewDelegate>
@property(nonatomic, copy) NSString *base;
@end

@implementation TBDocumentLinks

// A link annotation's URL, which PDFKit would otherwise give NSWorkspace.
- (void)PDFViewWillClickOnLink:(PDFView *)sender withURL:(NSURL *)url {
  tb_send_link(url, self.base);
}

// A link to another PDF file.
- (void)PDFViewOpenPDF:(PDFView *)sender forRemoteGoToAction:(PDFActionRemoteGoTo *)action {
  tb_send_link(action.URL, self.base);
}

@end

@interface TBDocumentView : PDFView
// Held here, as a view's delegate isn't kept by it.
@property(nonatomic, strong) TBDocumentLinks *links;
@end

@implementation TBDocumentView

- (BOOL)acceptsFirstMouse:(NSEvent *)event {
  return YES;
}

// Where a link's URL is, in an annotation that has one: a web or file link,
// or a link to another PDF file.
static NSURL *tb_link_of(PDFAnnotation *annotation) {
  if (annotation.URL) {
    return annotation.URL;
  }
  if ([annotation.action isKindOfClass:[PDFActionURL class]]) {
    return ((PDFActionURL *)annotation.action).URL;
  }
  if ([annotation.action isKindOfClass:[PDFActionRemoteGoTo class]]) {
    return ((PDFActionRemoteGoTo *)annotation.action).URL;
  }
  return nil;
}

// The link under a point in this view, if there's one.
- (NSURL *)linkAt:(NSPoint)inView {
  PDFPage *page = [self pageForPoint:inView nearest:NO];
  return page ? tb_link_of([page annotationAtPoint:[self convertPoint:inView toPage:page]]) : nil;
}

// A press on a link is this view's own, never PDFKit's inner views' (which
// would hand it to NSWorkspace): the release sends it to the gate.
- (NSView *)hitTest:(NSPoint)point {
  if (self.hidden) {
    return nil;
  }
  NSEventType type = NSApp.currentEvent.type;
  if ((type == NSEventTypeLeftMouseDown || type == NSEventTypeLeftMouseUp) && NSPointInRect(point, self.frame) && [self linkAt:[self convertPoint:point fromView:self.superview]]) {
    return self;
  }
  return [super hitTest:point];
}

- (void)mouseDown:(NSEvent *)event {
  if ([self linkAt:[self convertPoint:event.locationInWindow fromView:nil]]) {
    return;
  }
  [super mouseDown:event];
}

- (void)mouseUp:(NSEvent *)event {
  NSURL *link = [self linkAt:[self convertPoint:event.locationInWindow fromView:nil]];
  if (link) {
    tb_send_link(link, self.links.base);
    return;
  }
  [super mouseUp:event];
}

// Any action an annotation runs: within the document it's PDFKit's (going
// to a page, the named page moves, clearing a form); a web or file link goes
// to the gate; anything else does nothing.
- (void)performAction:(PDFAction *)action {
  if ([action isKindOfClass:[PDFActionGoTo class]] || [action isKindOfClass:[PDFActionNamed class]] || [action isKindOfClass:[PDFActionResetForm class]]) {
    [super performAction:action];
  } else if ([action isKindOfClass:[PDFActionURL class]]) {
    tb_send_link(((PDFActionURL *)action).URL, self.links.base);
  } else if ([action isKindOfClass:[PDFActionRemoteGoTo class]]) {
    tb_send_link(((PDFActionRemoteGoTo *)action).URL, self.links.base);
  }
}

@end

// All of a file's bytes from its open descriptor, or nil if it's too big or
// can't be read.
static NSData *tb_read_all(int fd, long long length) {
  if (length < 0 || length > TBDocumentLimit) {
    return nil;
  }
  NSMutableData *bytes = [NSMutableData dataWithLength:(NSUInteger)length];
  long long at = 0;
  while (at < length) {
    ssize_t got = pread(fd, (char *)bytes.mutableBytes + at, (size_t)(length - at), at);
    if (got <= 0) {
      return nil;
    }
    at += got;
  }
  return bytes;
}

// ---------------------------------------------------------------- the calls

// A view for the checked file open as `fd` (`length` bytes, at the real
// `path`) in `nativeParent` (a window's view), hidden until it's placed, or
// NULL if it can't be read as one. Owns `fd` from here, made or not. The
// caller owns the view and gives it back to tb_media_close. `muted` and
// `volume` are a review clip's, kept from the one shown before so clips
// compared by ear play at the same level; the pane's film always starts with
// its sound.
void *tb_media_open(void *nativeParent, int fd, long long length, const char *path, int kind, bool muted, float volume) {
  NSView *parent = (__bridge NSView *)nativeParent;
  NSString *file = path ? [NSString stringWithUTF8String:path] : nil;
  if (!parent || !file || fd < 0) {
    if (fd >= 0) {
      close(fd);
    }
    return NULL;
  }
  NSView *view = nil;
  if (kind == TBPaneDocument) {
    NSData *bytes = tb_read_all(fd, length);
    close(fd);
    PDFDocument *document = bytes ? [[PDFDocument alloc] initWithData:bytes] : nil;
    if (!document) {
      return NULL;
    }
    TBDocumentView *pdf = [[TBDocumentView alloc] initWithFrame:NSZeroRect];
    pdf.links = [TBDocumentLinks new];
    pdf.links.base = file.stringByDeletingLastPathComponent;
    pdf.delegate = pdf.links;
    pdf.autoScales = YES;
    pdf.displayMode = kPDFDisplaySinglePageContinuous;
    pdf.document = document;
    view = pdf;
  } else {
    TBFileSource *source = [[TBFileSource alloc] initWithDescriptor:fd length:length path:file];
    TBPlayerView *player = [[TBPlayerView alloc] initWithFrame:NSZeroRect];
    player.kind = kind;
    player.path = file;
    player.source = source;
    player.asset = tb_asset(source, file.lastPathComponent);
    player.videoGravity = AVLayerVideoGravityResizeAspect;
    player.controlsStyle = kind == TBCellVideo ? AVPlayerViewControlsStyleNone : AVPlayerViewControlsStyleFloating;
    // Live Text over a paused frame would take the press too.
    if (@available(macOS 13.0, *)) {
      player.allowsVideoFrameAnalysis = NO;
    }
    if (kind == TBPaneVideo) {
      // Paused on its first frame until its play button, as the Swift pane.
      AVPlayerItem *item = [AVPlayerItem playerItemWithAsset:player.asset];
      player.player = [AVPlayer playerWithPlayerItem:item];
      [player watchForFailure:item];
    } else {
      AVQueuePlayer *queue = [AVQueuePlayer new];
      queue.muted = muted;
      queue.volume = volume;
      // Looping: it mustn't keep the Mac awake on its own.
      queue.preventsDisplaySleepDuringVideoPlayback = NO;
      player.looper = [AVPlayerLooper playerLooperWithPlayer:queue templateItem:[AVPlayerItem playerItemWithAsset:player.asset]];
      player.player = queue;
      [player watchForFailure:player.looper];
      [queue play];
    }
    [player loadFilmSize];
    view = player;
  }
  view.hidden = YES;
  // GPUI places it every frame it's drawn; AppKit resizing it as well fights that.
  view.autoresizingMask = NSViewNotSizable;
  [parent addSubview:view];
  return (__bridge_retained void *)view;
}

void tb_media_place(void *nativeView, double x, double y, double width, double height) {
  NSView *view = (__bridge NSView *)nativeView;
  NSRect slot = NSMakeRect(x, y, width, height);
  if ([view isKindOfClass:[TBPlayerView class]]) {
    TBPlayerView *player = (TBPlayerView *)view;
    player.slot = slot;
    [player layoutInSlot];
  } else if (view) {
    tb_media_layout(view, slot, CGSizeZero);
  }
}

// Hidden while it isn't drawn where it is, or while a menu or a dialog is
// over it. A film hidden for a menu plays on, as it does under a Swift menu.
void tb_media_set_hidden(void *nativeView, bool hidden) {
  NSView *view = (__bridge NSView *)nativeView;
  if (view.hidden != hidden) {
    view.hidden = hidden;
  }
}

// Stops it and takes it out of its window for good, giving back the
// reference tb_media_open handed over. Its file closes once nothing's
// reading it.
void tb_media_close(void *nativeView) {
  NSView *view = (__bridge_transfer NSView *)nativeView;
  if ([view isKindOfClass:[TBPlayerView class]]) {
    TBPlayerView *player = (TBPlayerView *)view;
    [player stopWatching];
    [player.player pause];
    [player.looper disableLooping];
    player.looper = nil;
    player.player = nil;
  } else if ([view isKindOfClass:[TBDocumentView class]]) {
    ((TBDocumentView *)view).delegate = nil;
  }
  [view removeFromSuperview];
}

// The film's play button, for the scripted app. False when it isn't a film.
bool tb_media_play(void *nativeView) {
  NSView *view = (__bridge NSView *)nativeView;
  if (![view isKindOfClass:[TBPlayerView class]]) {
    return false;
  }
  [((TBPlayerView *)view).player play];
  return true;
}

void tb_media_state(void *nativeView, TBMediaState *out) {
  NSView *view = (__bridge NSView *)nativeView;
  *out = (TBMediaState){0};
  if ([view isKindOfClass:[TBPlayerView class]]) {
    TBPlayerView *player = (TBPlayerView *)view;
    out->width = player.filmSize.width;
    out->height = player.filmSize.height;
    out->playing = player.player.rate != 0;
    out->muted = player.player.isMuted;
    out->volume = player.player.volume;
    out->controls = player.controlsStyle != AVPlayerViewControlsStyleNone;
    out->looping = player.looper != nil;
    out->failed = player.failure != nil;
  } else if ([view isKindOfClass:[TBDocumentView class]]) {
    PDFView *pdf = (PDFView *)view;
    out->pages = (int)pdf.document.pageCount;
    PDFPage *first = out->pages > 0 ? [pdf.document pageAtIndex:0] : nil;
    if (first) {
      NSRect bounds = [first boundsForBox:kPDFDisplayBoxMediaBox];
      out->width = bounds.size.width;
      out->height = bounds.size.height;
    }
  }
}

// Why a film couldn't play, into `buffer` (cut to fit), or false if it's fine.
bool tb_media_failure(void *nativeView, char *buffer, int capacity) {
  NSView *view = (__bridge NSView *)nativeView;
  NSString *failure = [view isKindOfClass:[TBPlayerView class]] ? ((TBPlayerView *)view).failure : nil;
  if (!failure || capacity <= 0) {
    return false;
  }
  strlcpy(buffer, failure.UTF8String ?: "", (size_t)capacity);
  return true;
}

// The scripted app's click on a PDF's first link, as a person's would come:
// a press and a release at the link, through the window, so PDFKit handles
// it as it would a real one. False when there's no PDF or no link.
bool tb_media_click_link(void *nativeView) {
  NSView *view = (__bridge NSView *)nativeView;
  if (![view isKindOfClass:[TBDocumentView class]] || view.hidden || !view.window) {
    return false;
  }
  PDFView *pdf = (PDFView *)view;
  for (NSUInteger i = 0; i < pdf.document.pageCount; i++) {
    PDFPage *page = [pdf.document pageAtIndex:i];
    for (PDFAnnotation *annotation in page.annotations) {
      if (!annotation.URL && !annotation.action) {
        continue;
      }
      [pdf goToPage:page];
      NSRect box = annotation.bounds;
      NSPoint onPage = NSMakePoint(NSMidX(box), NSMidY(box));
      NSPoint inWindow = [pdf convertPoint:[pdf convertPoint:onPage fromPage:page] toView:nil];
      NSInteger number = pdf.window.windowNumber;
      for (NSNumber *type in @[ @(NSEventTypeLeftMouseDown), @(NSEventTypeLeftMouseUp) ]) {
        NSEvent *event = [NSEvent mouseEventWithType:(NSEventType)type.integerValue location:inWindow modifierFlags:0 timestamp:NSProcessInfo.processInfo.systemUptime windowNumber:number context:nil eventNumber:0 clickCount:1 pressure:1];
        [NSApp postEvent:event atStart:NO];
      }
      return true;
    }
  }
  return false;
}

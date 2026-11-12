// The Mac's own views for what GPUI can't draw: a video in AVKit's player and
// a PDF in PDFKit's view, each placed over a GPUI layout slot as the web
// runtime's pages are (desktop/web/native/appkit_hooks.m). media_view.rs
// places, hides and closes them. open_gate.rs is the only caller of
// tb_media_open, with the real file the gate has checked (decide_media), so
// nothing here decides what may be read.
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
} TBMediaState;

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

@interface TBPlayerView : AVPlayerView <NSDraggingSource>
@property(nonatomic) int kind;
@property(nonatomic) NSRect slot;
@property(nonatomic) CGSize filmSize;
@property(nonatomic, strong) AVPlayerLooper *looper;
@property(nonatomic, copy) NSURL *file;
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
  if (self.kind == TBPaneVideo && self.file && tb_is_drag(event)) {
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
// wanted to post a clip without opening the panel first, 29 Oct). The real
// file the gate checked, never whatever the path names by the time it's
// dropped. Copy only.
- (void)dragFile:(NSEvent *)event {
  if (![[NSFileManager defaultManager] fileExistsAtPath:self.file.path]) {
    return;
  }
  NSDraggingItem *item = [[NSDraggingItem alloc] initWithPasteboardWriter:self.file];
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
  AVURLAsset *asset = [AVURLAsset URLAssetWithURL:self.file options:nil];
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

@end

@interface TBDocumentView : PDFView
@end

@implementation TBDocumentView
- (BOOL)acceptsFirstMouse:(NSEvent *)event {
  return YES;
}
- (NSView *)hitTest:(NSPoint)point {
  return self.hidden ? nil : [super hitTest:point];
}
@end

// A view for the file at `path` in `nativeParent` (a window's view), hidden
// until it's placed, or NULL if the file can't be read as one. The caller
// owns the view and gives it back to tb_media_close. `muted` and `volume`
// are a review clip's, kept from the one shown before so clips compared by
// ear play at the same level; the pane's film always starts with its sound.
void *tb_media_open(void *nativeParent, const char *path, int kind, bool muted, float volume) {
  NSView *parent = (__bridge NSView *)nativeParent;
  if (!parent || !path) {
    return NULL;
  }
  NSString *file = [NSString stringWithUTF8String:path];
  if (!file) {
    return NULL;
  }
  NSURL *url = [NSURL fileURLWithPath:file];
  NSView *view = nil;
  if (kind == TBPaneDocument) {
    PDFDocument *document = [[PDFDocument alloc] initWithURL:url];
    if (!document) {
      return NULL;
    }
    TBDocumentView *pdf = [[TBDocumentView alloc] initWithFrame:NSZeroRect];
    pdf.autoScales = YES;
    pdf.displayMode = kPDFDisplaySinglePageContinuous;
    pdf.document = document;
    view = pdf;
  } else {
    TBPlayerView *player = [[TBPlayerView alloc] initWithFrame:NSZeroRect];
    player.kind = kind;
    player.file = url;
    player.videoGravity = AVLayerVideoGravityResizeAspect;
    player.controlsStyle = kind == TBCellVideo ? AVPlayerViewControlsStyleNone : AVPlayerViewControlsStyleFloating;
    // Live Text over a paused frame would take the press too.
    if (@available(macOS 13.0, *)) {
      player.allowsVideoFrameAnalysis = NO;
    }
    if (kind == TBPaneVideo) {
      // Paused on its first frame until its play button, as the Swift pane.
      player.player = [AVPlayer playerWithURL:url];
    } else {
      AVQueuePlayer *queue = [AVQueuePlayer new];
      queue.muted = muted;
      queue.volume = volume;
      // Looping: it mustn't keep the Mac awake on its own.
      queue.preventsDisplaySleepDuringVideoPlayback = NO;
      player.looper = [AVPlayerLooper playerLooperWithPlayer:queue templateItem:[AVPlayerItem playerItemWithURL:url]];
      player.player = queue;
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
// reference tb_media_open handed over.
void tb_media_close(void *nativeView) {
  NSView *view = (__bridge_transfer NSView *)nativeView;
  if ([view isKindOfClass:[TBPlayerView class]]) {
    TBPlayerView *player = (TBPlayerView *)view;
    [player.player pause];
    [player.looper disableLooping];
    player.looper = nil;
    player.player = nil;
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

// The AppKit glue for running CEF inside the GPUI app on a Mac. Adapted from
// Ghostex's apps/desktop/native/macos/GpuiCefAppKitHooks.m by way of the
// gpui-web-spike branch (desktop/web-spike/native/appkit_hooks.m there), MIT
// licensed:
//
//   Copyright 2026 Mohamad Yahia (maddada)
//
//   Permission is hereby granted, free of charge, to any person obtaining a
//   copy of this software and associated documentation files (the "Software"),
//   to deal in the Software without restriction, including without limitation
//   the rights to use, copy, modify, merge, publish, distribute, sublicense,
//   and/or sell copies of the Software, and to permit persons to whom the
//   Software is furnished to do so, subject to the following conditions:
//
//   The above copyright notice and this permission notice shall be included in
//   all copies or substantial portions of the Software.
//
//   THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//   IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//   FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//   AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//   LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
//   FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//   DEALINGS IN THE SOFTWARE.
#import <AppKit/AppKit.h>
#import <objc/runtime.h>
#include <stdbool.h>

// CEF's message pump traps unless NSApp conforms to CefAppProtocol
// (cef_application_mac.h). GPUI owns the NSApplication subclass, so the
// protocol and the isHandlingSendEvent state are added at run time instead.
@protocol CrAppProtocol
- (BOOL)isHandlingSendEvent;
@end
@protocol CrAppControlProtocol <CrAppProtocol>
- (void)setHandlingSendEvent:(BOOL)handlingSendEvent;
@end
@protocol CefAppProtocol <CrAppControlProtocol>
@end

static BOOL g_handlingSendEvent = NO;

@interface NSApplication (ThreadbearCEF) <CefAppProtocol>
- (BOOL)isHandlingSendEvent;
- (void)setHandlingSendEvent:(BOOL)handlingSendEvent;
- (void)threadbearCefSendEvent:(NSEvent *)event;
@end

@implementation NSApplication (ThreadbearCEF)
- (BOOL)isHandlingSendEvent {
  return g_handlingSendEvent;
}
- (void)setHandlingSendEvent:(BOOL)handlingSendEvent {
  g_handlingSendEvent = handlingSendEvent;
}
- (void)threadbearCefSendEvent:(NSEvent *)event {
  BOOL was = g_handlingSendEvent;
  g_handlingSendEvent = YES;
  // Swizzled: this calls the original -sendEvent:.
  [self threadbearCefSendEvent:event];
  g_handlingSendEvent = was;
}
@end

void tb_cef_install_app_hooks(void) {
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    Class baseClass = [NSApplication class];
    Method original = class_getInstanceMethod(baseClass, @selector(sendEvent:));
    Method swapped = class_getInstanceMethod(baseClass, @selector(threadbearCefSendEvent:));
    if (original && swapped) {
      method_exchangeImplementations(original, swapped);
    }
    class_addProtocol(baseClass, @protocol(CrAppProtocol));
    class_addProtocol(baseClass, @protocol(CrAppControlProtocol));
    class_addProtocol(baseClass, @protocol(CefAppProtocol));
  });
  // NSApp is GPUI's subclass; Chromium checks the concrete class.
  if (NSApp) {
    Class appClass = object_getClass(NSApp);
    class_addProtocol(appClass, @protocol(CrAppProtocol));
    class_addProtocol(appClass, @protocol(CrAppControlProtocol));
    class_addProtocol(appClass, @protocol(CefAppProtocol));
  }
}

// Implemented in Rust (host.rs): starts CEF, then runs Chromium's message
// loop for the rest of the process.
extern void tb_cef_run_message_loop(void);

static BOOL g_startPending = NO;
static BOOL g_loopRunning = NO;

static void tb_cef_schedule_start(CFTimeInterval delay) {
  CFRunLoopTimerRef timer = CFRunLoopTimerCreateWithHandler(
      kCFAllocatorDefault, CFAbsoluteTimeGetCurrent() + delay, 0, 0, 0,
      ^(CFRunLoopTimerRef firedTimer) {
        (void)firedTimer;
        if (!g_startPending) {
          return;
        }
        // Not inside a modal session (an alert, a save panel): Chromium's
        // loop would end up nested in it.
        if (NSApp.modalWindow != nil) {
          tb_cef_schedule_start(0.25);
          return;
        }
        g_startPending = NO;
        g_loopRunning = YES;
        tb_cef_run_message_loop();
        g_loopRunning = NO;
      });
  CFRunLoopAddTimer(CFRunLoopGetMain(), timer, kCFRunLoopDefaultMode);
  CFRelease(timer);
}

// Why a CFRunLoopTimer and never a main-queue block: Chromium's loop then
// runs inside this one callout for the rest of the process, and it has to be
// entered from a run-loop pass rather than a GCD block, or GPUI's main-queue
// work (every foreground task) would stop being serviced (Ghostex's note on
// this, 2026-10-17; the spike found the same).
void tb_cef_start_message_loop(void) {
  if (g_startPending || g_loopRunning) {
    return;
  }
  g_startPending = YES;
  tb_cef_schedule_start(0);
  CFRunLoopWakeUp(CFRunLoopGetMain());
}

// A page's native view placed in a GPUI layout slot. GPUI's coordinates are
// the window's, from the top left, in points; the parent view may not be
// flipped, so the y is turned over.
void tb_view_set_frame(void *nativeView, double x, double y, double width, double height) {
  NSView *view = (__bridge NSView *)nativeView;
  if (!view) {
    return;
  }
  // GPUI places it every frame; AppKit resizing it as well fights that.
  view.autoresizingMask = NSViewNotSizable;
  NSView *parent = view.superview;
  CGFloat nativeY = y;
  if (parent && !parent.isFlipped) {
    nativeY = NSHeight(parent.bounds) - y - height;
  }
  view.frame = NSMakeRect(x, nativeY, MAX(0.0, width), MAX(0.0, height));
}

void tb_view_set_hidden(void *nativeView, bool hidden) {
  NSView *view = (__bridge NSView *)nativeView;
  view.hidden = hidden;
}

// The app's bundle id, for the helpers' ids, or NULL when it isn't running
// from a bundle (a cargo build the specs start). The string is the bundle's
// and lives as long as it.
const char *tb_main_bundle_identifier(void) {
  NSString *identifier = [[NSBundle mainBundle] bundleIdentifier];
  return identifier ? identifier.UTF8String : NULL;
}

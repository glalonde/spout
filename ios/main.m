// This file is a placeholder so Xcode has a valid source to compile and link.
// The "Build Rust Binary" run script phase replaces the linked binary with the
// Rust executable before Xcode signs the app bundle. The Rust binary provides
// the real main() which uses winit to drive UIApplicationMain internally.
#import <UIKit/UIKit.h>

@interface SpoutPlaceholderDelegate : UIResponder <UIApplicationDelegate>
@end
@implementation SpoutPlaceholderDelegate
@end

int main(int argc, char * argv[]) {
    return UIApplicationMain(argc, argv, nil,
        NSStringFromClass([SpoutPlaceholderDelegate class]));
}

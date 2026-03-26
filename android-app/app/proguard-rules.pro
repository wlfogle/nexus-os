# TiamatsStack ProGuard rules
# Keep WebView JavaScript interfaces
-keepclassmembers class * {
    @android.webkit.JavascriptInterface <methods>;
}
# Keep all public classes in our package
-keep public class com.tiamat.mediastack.** { *; }

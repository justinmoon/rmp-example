import SwiftUI

@main
struct HelloChatApp: App {
    @State private var manager = AppManager()

    var body: some Scene {
        WindowGroup {
            ContentView(manager: manager)
        }
    }
}

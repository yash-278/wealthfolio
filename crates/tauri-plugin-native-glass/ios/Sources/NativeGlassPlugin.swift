import SwiftUI
import Tauri
import WebKit

struct GlassItem: Decodable, Identifiable {
    let id: String
    let title: String
    let symbol: String
    let selected: Bool
    let children: [GlassItem]?
}

struct GlassControls: Decodable {
    let id: String
    let x: Double
    let y: Double
    let width: Double
    let height: Double
    let visible: Bool
    let dark: Bool
    let labels: Bool
    let items: [GlassItem]
}

struct RemoveControls: Decodable { let id: String }

@available(iOS 26.0, *)
private struct GlassBar: View {
    let controls: GlassControls
    let action: (String) -> Void
    @Namespace private var selection
    @State private var selectedID: String?
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    init(controls: GlassControls, action: @escaping (String) -> Void) {
        self.controls = controls
        self.action = action
        _selectedID = State(initialValue: controls.items.first(where: { $0.selected })?.id)
    }

    private func select(_ id: String) {
        withAnimation(reduceMotion ? nil : .snappy(duration: 0.28)) { selectedID = id }
    }

    private func label(_ item: GlassItem) -> some View {
        Group {
            if controls.labels {
                VStack(spacing: 3) {
                    Image(systemName: item.symbol).font(.system(size: 21, weight: .medium))
                    Text(item.title).font(.caption2).lineLimit(1)
                }
            } else {
                HStack(spacing: 5) {
                    Image(systemName: item.symbol).font(.system(size: 18, weight: .medium))
                    if selectedID == item.id {
                        Text(item.title).font(.subheadline.weight(.medium)).lineLimit(1)
                    }
                }
            }
        }
        .frame(minWidth: 36, maxWidth: controls.labels || selectedID == item.id ? .infinity : nil,
               maxHeight: .infinity)
        .contentShape(Capsule())
        .accessibilityLabel(item.title)
    }

    var body: some View {
        GlassEffectContainer(spacing: 8) {
            HStack(spacing: 2) {
                ForEach(controls.items) { item in
                    if let children = item.children, !children.isEmpty {
                        Menu {
                            ForEach(children) { child in
                                Button { action(child.id) } label: {
                                    Label(child.title, systemImage: child.symbol)
                                }
                            }
                        } label: { label(item) }
                        .buttonStyle(.plain)
                    } else {
                        Button {
                            if controls.labels || controls.items.contains(where: { $0.selected }) {
                                select(item.id)
                            }
                            action(item.id)
                        } label: {
                            label(item)
                                .background {
                                    if selectedID == item.id {
                                        Capsule().fill(.clear)
                                            .glassEffect(.regular.interactive(), in: .capsule)
                                            .matchedGeometryEffect(id: "selection", in: selection)
                                    }
                                }
                        }
                        .buttonStyle(.plain)
                        .accessibilityAddTraits(selectedID == item.id ? .isSelected : [])
                    }
                }
            }
            .padding(4)
            .glassEffect(.regular, in: .capsule)
        }
        .onChange(of: controls.items.first(where: { $0.selected })?.id) { _, value in
            if value != selectedID { select(value ?? "") }
        }
        .foregroundStyle(.primary)
        .environment(\.colorScheme, controls.dark ? .dark : .light)
    }
}

/// Each host occupies only its control rectangle, so content beneath remains scrollable.
class NativeGlassPlugin: Plugin {
    private weak var webview: WKWebView?
    private var hosts: [String: UIViewController] = [:]

    override func load(webview: WKWebView) {
        self.webview = webview
    }

    @objc func update(_ invoke: Invoke) throws {
        let controls = try invoke.parseArgs(GlassControls.self)
        DispatchQueue.main.async { [weak self] in
            guard let self, let webview = self.webview,
                  let parent = webview.superview else {
                invoke.reject("Web view is not ready")
                return
            }
            guard #available(iOS 26.0, *) else {
                invoke.resolve(["supported": false])
                return
            }
            let root = GlassBar(controls: controls) { [weak self] item in
                UISelectionFeedbackGenerator().selectionChanged()
                self?.trigger("action", data: ["control": controls.id, "item": item])
            }
            let host: UIHostingController<GlassBar>
            if let existing = self.hosts[controls.id] as? UIHostingController<GlassBar> {
                host = existing
                host.rootView = root
            } else {
                host = UIHostingController(rootView: root)
                host.view.backgroundColor = .clear
                host.safeAreaRegions = []
                var responder: UIResponder? = webview
                while let current = responder, !(current is UIViewController) {
                    responder = current.next
                }
                if let owner = responder as? UIViewController {
                    owner.addChild(host)
                    parent.addSubview(host.view)
                    host.didMove(toParent: owner)
                } else {
                    invoke.reject("Web view controller is not ready")
                    return
                }
                self.hosts[controls.id] = host
            }
            host.view.frame = webview.convert(CGRect(x: controls.x, y: controls.y,
                width: controls.width, height: controls.height), to: parent)
            host.view.isHidden = !controls.visible
            parent.bringSubviewToFront(host.view)
            invoke.resolve(["supported": true])
        }
    }

    @objc func remove(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(RemoveControls.self)
        DispatchQueue.main.async { [weak self] in
            if let host = self?.hosts.removeValue(forKey: args.id) {
                host.willMove(toParent: nil)
                host.view.removeFromSuperview()
                host.removeFromParent()
            }
            invoke.resolve()
        }
    }
}

@_cdecl("init_plugin_native_glass")
func initPlugin() -> Plugin { NativeGlassPlugin() }

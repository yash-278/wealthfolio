import SwiftUI

struct ActivitiesView: View {
    @Environment(AppModel.self) private var model
    @Environment(\.colorScheme) private var scheme
    @Environment(\.dynamicTypeSize) private var textSize
    var accountID: String?
    @State private var records: [Record] = []
    @State private var query = ""
    @State private var selectedAccount = ""
    @State private var page = 0
    @State private var more = false
    @State private var loading = false
    @State private var requestID = UUID()
    @State private var editing: Record?
    @State private var adding = false
    @State private var deleting: Record?
    @State private var error: String?
    private var accent: Color { scheme == .dark ? Color(red: 0.62, green: 0.86, blue: 0.83) : Color(red: 0.16, green: 0.46, blue: 0.44) }
    private var canvas: Color { scheme == .dark ? Color(red: 0.067, green: 0.106, blue: 0.125) : Color(red: 0.96, green: 0.975, blue: 0.98) }
    private var scope: String { accountID ?? selectedAccount }
    private var scopeName: String { model.accounts.first(where: { $0.id == scope })?["name"].text ?? "All accounts" }
    private struct DayGroup: Identifiable { let id: String; let title: String; var rows: [Record] }
    private var groups: [DayGroup] {
        let key = DateFormatter(); key.locale = Locale(identifier: "en_US_POSIX"); key.calendar = Calendar(identifier: .gregorian)
        key.timeZone = TimeZone(identifier: model.settings["timezone"].text) ?? .current; key.dateFormat = "yyyy-MM-dd"
        let label = DateFormatter(); label.timeZone = key.timeZone; label.dateStyle = .medium
        var result: [DayGroup] = []
        for row in records {
            let date = parseActivityDate(row["date"].text)
            let id = date.map(key.string) ?? "unknown"
            if let index = result.firstIndex(where: { $0.id == id }) { result[index].rows.append(row) }
            else { result.append(DayGroup(id: id, title: date.map(label.string) ?? "Date unavailable", rows: [row])) }
        }
        return result
    }
    var body: some View {
        List {
            Section {
                HStack {
                    VStack(alignment: .leading, spacing: 6) {
                        Text(scopeName).font(.subheadline.weight(.medium))
                        Text(loading ? "Updating transactions…" : "\(records.count) transactions loaded\(more ? " · more available" : "")")
                            .font(.caption).foregroundStyle(.secondary)
                    }
                    Spacer()
                    if loading { ProgressView().controlSize(.small) }
                }.padding(.vertical, 2)
            }.listRowBackground(Color.clear).listRowSeparator(.hidden)
            if let error {
                Section { Label(error, systemImage: "exclamationmark.circle").font(.subheadline).foregroundStyle(.red)
                    Button("Try again") { Task { await load(reset: true) } }
                }.listRowBackground(Color.clear)
            }
            ForEach(groups) { group in
                Section {
                    ForEach(group.rows) { activity in
                        Button { editing = activity } label: { activityRow(activity) }
                            .buttonStyle(.plain)
                            .listRowInsets(EdgeInsets(top: 6, leading: 16, bottom: 6, trailing: 16))
                            .listRowBackground(Color.clear).listRowSeparator(.hidden)
                            .swipeActions { Button("Delete", role: .destructive) { deleting = activity } }
                            .contextMenu {
                                Button("Edit transaction", systemImage: "pencil") { editing = activity }
                                Button("Delete transaction", systemImage: "trash", role: .destructive) { deleting = activity }
                            }
                    }
                } header: { Text(group.title).font(.subheadline.weight(.semibold)).foregroundStyle(Color.primary.opacity(0.7)).textCase(nil) }
            }
            if more {
                Section { Button { Task { await load(reset: false) } } label: {
                    HStack { Spacer(); Text(loading ? "Loading…" : "Load more transactions"); Spacer() }.font(.subheadline).frame(minHeight: 44)
                }.buttonStyle(.glass).disabled(loading) }.listRowBackground(Color.clear).listRowSeparator(.hidden)
            }
            if records.isEmpty && !loading && error == nil {
                ContentUnavailableView(query.isEmpty ? "No transactions yet" : "No matching transactions", systemImage: "list.bullet.rectangle",
                    description: Text(query.isEmpty ? "Add a transaction or connect to your server to bring in your activity." : "Try a different asset or account."))
                    .listRowBackground(Color.clear).listRowSeparator(.hidden)
            }
        }
        .listStyle(.plain).listSectionSpacing(.compact).environment(\.defaultMinListHeaderHeight, 32).scrollContentBackground(.hidden).background(canvas).tint(accent)
        .leadingPageTitle("Activities")
        .searchable(text: $query, prompt: "Search by asset")
        .toolbar {
            if accountID == nil {
                ToolbarItem(placement: .topBarTrailing) {
                    Menu("Filter accounts", systemImage: selectedAccount.isEmpty ? "line.3.horizontal.decrease" : "line.3.horizontal.decrease.circle.fill") {
                        Picker("Account", selection: $selectedAccount) {
                            Text("All accounts").tag("")
                            ForEach(model.accounts) { Text($0["name"].text).tag($0.id) }
                        }
                    }
                }
            }
            ToolbarItem(placement: .topBarTrailing) { Button("Add transaction", systemImage: "plus") { adding = true } }
        }
        .task(id: "\(model.revision)|\(query)|\(scope)") {
            do { try await Task.sleep(for: .milliseconds(250)) } catch { return }
            await load(reset: true)
        }
        .refreshable { await load(reset: true) }
        .sheet(item: $editing) { record in NavigationStack { ActivityEditor(activity: record) } }
        .sheet(isPresented: $adding) { NavigationStack { ActivityEditor(defaultAccountID: scope.isEmpty ? nil : scope) } }
        .confirmationDialog("Delete this transaction?", isPresented: Binding(get: { deleting != nil }, set: { if !$0 { deleting = nil } })) {
            Button("Delete transaction", role: .destructive) {
                guard let record = deleting else { return }
                Task { do { _ = try await model.mutate("/api/v1/activities/" + record.id, method: "DELETE") } catch { model.error = error.localizedDescription }; deleting = nil }
            }
        }
    }
    private func activityRow(_ activity: Record) -> some View {
        let type = activity["activityType"].text
        let tint = transactionColor(type)
        let title = !activity["assetSymbol"].text.isEmpty ? activity["assetSymbol"].text : (!activity["comment"].text.isEmpty ? activity["comment"].text : type.replacingOccurrences(of: "_", with: " ").capitalized)
        let layout = textSize.isAccessibilitySize ? AnyLayout(VStackLayout(alignment: .leading, spacing: 12)) : AnyLayout(HStackLayout(alignment: .center, spacing: 12))
        return layout {
            HStack(spacing: 12) {
                Image(systemName: symbol(type)).font(.body).foregroundStyle(tint)
                    .frame(width: 40, height: 40).background(tint.opacity(scheme == .dark ? 0.14 : 0.09), in: .rect(cornerRadius: 12)).accessibilityHidden(true)
                VStack(alignment: .leading, spacing: 6) {
                    Text(title).font(.body.weight(.medium)).lineLimit(textSize.isAccessibilitySize ? nil : 2)
                    Text(activity["accountName"].text).font(.caption).foregroundStyle(.secondary)
                }
            }.frame(maxWidth: .infinity, alignment: .leading)
            VStack(alignment: textSize.isAccessibilitySize ? .leading : .trailing, spacing: 6) {
                Text(money(activity["amount"], currency: activity["currency"].text, hidden: model.hideBalances)).font(.subheadline.weight(.semibold)).monospacedDigit()
                Text(type.replacingOccurrences(of: "_", with: " ").capitalized).font(.caption).foregroundStyle(tint)
            }.fixedSize(horizontal: !textSize.isAccessibilitySize, vertical: false)
        }.foregroundStyle(.primary).padding(.vertical, 6).contentShape(.rect)
    }
    // Type labels and symbols preserve meaning independently of color.
    private func transactionColor(_ type: String) -> Color {
        let dark = scheme == .dark
        switch type {
        case "DEPOSIT", "CREDIT", "DIVIDEND", "INTEREST":
            return dark ? Color(red: 0.49, green: 0.84, blue: 0.66) : Color(red: 0.12, green: 0.43, blue: 0.29)
        case "WITHDRAWAL", "FEE", "TAX":
            return dark ? Color(red: 1, green: 0.66, blue: 0.57) : Color(red: 0.66, green: 0.25, blue: 0.18)
        case "BUY":
            return dark ? Color(red: 0.55, green: 0.76, blue: 1) : Color(red: 0.16, green: 0.38, blue: 0.66)
        case "SELL":
            return dark ? Color(red: 0.99, green: 0.78, blue: 0.43) : Color(red: 0.53, green: 0.35, blue: 0.08)
        case "TRANSFER_IN", "TRANSFER_OUT":
            return dark ? Color(red: 0.76, green: 0.68, blue: 0.97) : Color(red: 0.45, green: 0.32, blue: 0.68)
        default: return accent
        }
    }
    private func symbol(_ type: String) -> String {
        switch type {
        case "DEPOSIT", "TRANSFER_IN", "CREDIT": "arrow.down.left"
        case "WITHDRAWAL", "TRANSFER_OUT": "arrow.up.right"
        case "BUY": "cart"
        case "SELL": "tag"
        case "DIVIDEND", "INTEREST": "plus"
        case "FEE", "TAX": "receipt"
        default: "doc.text"
        }
    }
    private func load(reset: Bool) async {
        guard reset || !loading else { return }
        let token = UUID(); requestID = token; loading = true
        defer { if requestID == token { loading = false } }
        let next = reset ? 0 : page + 1
        var body: [String: JSONValue] = ["page": .number(Decimal(next)), "pageSize": .number(50), "sort": .object(["id": .string("date"), "desc": .bool(true)])]
        if !scope.isEmpty { body["accountIdFilter"] = .string(scope) }
        if !query.isEmpty { body["assetIdKeyword"] = .string(query) }
        do {
            let result = try await model.engine.request("/api/v1/activities/search", method: "POST", body: .object(body))
            guard requestID == token, !Task.isCancelled else { return }
            let rows = result["data"].values.map(Record.init)
            records = reset ? rows : records + rows; page = next; more = rows.count == 50; error = nil
        } catch { if requestID == token && !Task.isCancelled { self.error = error.localizedDescription } }
    }
}

struct ActivityEditor: View {
    @Environment(AppModel.self) private var model
    @Environment(\.dismiss) private var dismiss
    @Environment(\.colorScheme) private var scheme
    @State private var showDetails = false
    var activity: Record?
    var defaultAccountID: String?
    @State private var accountID = ""
    @State private var type = "DEPOSIT"
    @State private var amount = ""
    @State private var quantity = ""
    @State private var price = ""
    @State private var fee = ""
    @State private var tax = ""
    @State private var symbol = ""
    @State private var currency = ""
    @State private var notes = ""
    @State private var date = Date()
    @State private var dateValid = true
    @State private var saving = false
    @State private var error: String?
    var body: some View {
        Form {
            Section("Transaction") {
                Picker("Account", selection: $accountID) { ForEach(model.accounts) { Text($0["name"].text).tag($0.id) } }
                Picker("Type", selection: $type) { ForEach(["BUY", "SELL", "DIVIDEND", "INTEREST", "DEPOSIT", "WITHDRAWAL", "TRANSFER_IN", "TRANSFER_OUT", "FEE", "TAX", "SPLIT", "CREDIT", "ADJUSTMENT"], id: \.self) { Text($0.replacingOccurrences(of: "_", with: " ").capitalized).tag($0) } }
                DatePicker("Date", selection: $date)
            }
            Section("Value") {
                LabeledContent("Currency") { TextField("INR", text: $currency).multilineTextAlignment(.trailing).textInputAutocapitalization(.characters).autocorrectionDisabled() }
                LabeledContent("Amount") { TextField("0.00", text: $amount).keyboardType(.decimalPad).multilineTextAlignment(.trailing).monospacedDigit() }
            }
            Section {
                DisclosureGroup("Asset, quantity and charges", isExpanded: $showDetails) {
                    LabeledContent("Asset") { TextField("Symbol", text: $symbol).multilineTextAlignment(.trailing).textInputAutocapitalization(.characters).autocorrectionDisabled() }
                    LabeledContent("Quantity") { TextField("Optional", text: $quantity).keyboardType(.decimalPad).multilineTextAlignment(.trailing) }
                    LabeledContent("Unit price") { TextField("Optional", text: $price).keyboardType(.decimalPad).multilineTextAlignment(.trailing) }
                    LabeledContent("Fee") { TextField("Optional", text: $fee).keyboardType(.decimalPad).multilineTextAlignment(.trailing) }
                    LabeledContent("Tax") { TextField("Optional", text: $tax).keyboardType(.decimalPad).multilineTextAlignment(.trailing) }
                }
            }
            Section("Notes") { TextField("Add a note", text: $notes, axis: .vertical).lineLimit(3...6) }
            if let error { Text(error).foregroundStyle(.red) }
        }.navigationTitle(activity == nil ? "New transaction" : "Edit transaction")
        .navigationBarTitleDisplayMode(.inline)
        .scrollContentBackground(.hidden)
        .background(scheme == .dark ? Color(red: 0.067, green: 0.106, blue: 0.125) : Color(red: 0.96, green: 0.975, blue: 0.98))
        .toolbar {
            ToolbarItem(placement: .cancellationAction) { Button("Cancel") { dismiss() } }
            ToolbarItem(placement: .confirmationAction) { Button(saving ? "Saving…" : "Save") { Task { await save() } }.disabled(saving || accountID.isEmpty || !dateValid) }
        }.onChange(of: type) { _, value in if ["BUY", "SELL", "SPLIT"].contains(value) { showDetails = true } }
        .onAppear {
            accountID = activity?["accountId"].text ?? defaultAccountID ?? model.accounts.first?.id ?? ""
            type = activity?["activityType"].text ?? "DEPOSIT"
            amount = activity?["amount"].text ?? ""; quantity = activity?["quantity"].text ?? ""
            price = activity?["unitPrice"].text ?? ""; fee = activity?["fee"].text ?? ""; tax = activity?["tax"].text ?? ""
            symbol = activity?["assetSymbol"].text ?? ""; currency = activity?["currency"].text ?? model.accounts.first(where: { $0.id == accountID })?["currency"].text ?? model.currency
            notes = activity?["comment"].text ?? ""
            showDetails = ["BUY", "SELL", "SPLIT"].contains(type) || !symbol.isEmpty || !quantity.isEmpty || !fee.isEmpty || !tax.isEmpty
            if let text = activity?["date"].text {
                if let parsed = parseActivityDate(text) { date = parsed }
                else { dateValid = false; error = "The transaction date could not be read. No changes have been saved." }
            }
        }
    }
    private func save() async {
        saving = true; defer { saving = false }
        var body: [String: JSONValue] = ["accountId": .string(accountID), "activityType": .string(type),
            "activityDate": .string(ISO8601DateFormatter().string(from: date)), "currency": .string(currency), "notes": .string(notes)]
        for (key, value) in [("amount", amount), ("quantity", quantity), ("unitPrice", price), ("fee", fee), ("tax", tax)] {
            body[key] = value.isEmpty ? .null : .string(value)
        }
        if let activity { body["id"] = .string(activity.id) }
        else { body["sourceSystem"] = .string("MANUAL") }
        if !symbol.isEmpty && symbol != activity?["assetSymbol"].text {
            body["asset"] = .object(["key": .string(symbol), "sourceSymbol": .string(symbol),
                "accountCurrency": .string(model.accounts.first(where: { $0.id == accountID })?["currency"].text ?? currency)])
        }
        do { _ = try await model.mutate("/api/v1/activities", method: activity == nil ? "POST" : "PUT", body: .object(body)); dismiss() }
        catch { self.error = error.localizedDescription }
    }
}

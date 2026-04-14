# Kesh — Messages English (Switzerland)

# Authentication errors
error-invalid-credentials = Invalid credentials
error-unauthenticated = Not authenticated
error-invalid-refresh-token = Session expired
error-rate-limited = Too many attempts

# Authorization errors
error-forbidden = Access denied
error-cannot-disable-self = Cannot disable your own account
error-cannot-disable-last-admin = Cannot disable the last administrator

# Resource errors
error-not-found = Resource not found
error-conflict = Resource already exists
error-optimistic-lock = Version conflict — the resource has been modified
error-foreign-key = Invalid reference
error-check-constraint = Invalid value
error-illegal-state = Illegal state transition

# Validation errors
error-validation = Validation error
error-username-empty = Username cannot be empty
error-username-too-long = Username must not exceed { $max } characters

# System errors
error-internal = Internal error
error-service-unavailable = Service temporarily unavailable

# Onboarding errors (Story 2.2)
error-onboarding-step-already-completed = This configuration step has already been completed

# Onboarding — wizard
onboarding-choose-mode = Choose your usage mode
onboarding-mode-guided = Guided
onboarding-mode-guided-desc = Generous spacing, contextual help, confirmations before actions
onboarding-mode-expert = Expert
onboarding-mode-expert-desc = Compact interface, keyboard shortcuts, direct actions
onboarding-choose-path = How would you like to start?
onboarding-path-demo = Explore with demo data
onboarding-path-demo-desc = Discover Kesh with realistic sample data
onboarding-path-production = Set up for production
onboarding-path-production-desc = Configure your organisation to start working

# Demo banner
demo-banner-text = Demo instance — fictitious data
demo-banner-reset = Reset for production
demo-reset-confirm-title = Reset instance
demo-reset-confirm-body = All demo data will be deleted. Do you want to continue?
demo-reset-confirm-ok = Confirm
demo-reset-confirm-cancel = Cancel

# Onboarding — Path B (Story 2.3)
onboarding-choose-org-type = Organisation type
onboarding-org-independant = Independent
onboarding-org-independant-desc = Self-employed, freelancer
onboarding-org-association = Association
onboarding-org-association-desc = Non-profit association
onboarding-org-pme = SME
onboarding-org-pme-desc = Small and medium enterprise (Ltd, LLC)
onboarding-choose-accounting-lang = Accounting language
onboarding-accounting-lang-desc = Language of chart of accounts labels (independent of interface language)
onboarding-coordinates-title = Your organisation details
onboarding-field-name = Name / Company name
onboarding-field-address = Address
onboarding-field-ide = UID number
onboarding-field-ide-hint = optional, format CHE-xxx.xxx.xxx
onboarding-bank-title = Primary bank account
onboarding-field-bank-name = Bank name
onboarding-field-iban = IBAN
onboarding-field-qr-iban = QR-IBAN
onboarding-skip-bank = Configure later
onboarding-next = Continue
incomplete-banner-text = Configuration incomplete — Complete setup
incomplete-banner-cta = Complete setup

# Homepage (Story 2.4)
homepage-title = Dashboard
homepage-entries-title = Recent entries
homepage-entries-empty = No entries.
homepage-entries-empty-guided = No entries yet. Start by recording your first journal entry.
homepage-entries-action = Record an entry
homepage-invoices-title = Open invoices
homepage-invoices-empty = No open invoices.
homepage-invoices-empty-guided = No open invoices. Create your first invoice to bill your clients.
homepage-invoices-action = Create an invoice
homepage-bank-title = Bank accounts
homepage-bank-empty = No bank account.
homepage-bank-empty-guided = No bank account configured. Add your account to import statements.
homepage-bank-no-transactions = No imported transactions
homepage-bank-action = Configure

# Settings (Story 2.4)
settings-title = Settings
settings-org-title = Organisation
settings-accounting-title = Accounting
settings-bank-title = Bank accounts
settings-users-title = Users
settings-field-name = Name
settings-field-address = Address
settings-field-ide = UID
settings-field-org-type = Organisation type
settings-field-instance-language = Interface language
settings-field-accounting-language = Accounting language
search-coming-soon = Search coming soon

# Misc i18n (Story 2.4 review)
loading = Loading...
settings-edit = Edit
settings-edit-coming-soon = Editing coming soon
settings-manage = Manage
settings-no-bank = No bank account configured.
settings-no-company = No organisation configured. Complete onboarding first.

# Chart of Accounts (Story 3.1)
accounts-title = Chart of accounts
accounts-add = New account
accounts-edit = Edit account
accounts-archive = Archive
accounts-archive-confirm = The account will no longer be available in future selections, but will remain visible in existing entries.
account-field-number = Number
account-field-name = Name
account-field-type = Type
account-field-parent = Parent account
account-type-asset = Asset
account-type-liability = Liability
account-type-revenue = Revenue
account-type-expense = Expense
account-archived-label = Archived

# Mode Guided/Expert (Story 2.5)
mode-guided-label = Guided
mode-expert-label = Expert
shortcut-new-entry = Ctrl+N : New entry

# Journal entries (Story 3.2)
error-entry-unbalanced = Unbalanced entry — total debits ({ $debit }) does not match total credits ({ $credit })
error-no-fiscal-year = No fiscal year exists for date { $date }. Create a fiscal year before recording entries.
error-fiscal-year-closed = The fiscal year for date { $date } is closed — no entries can be added or modified (Swiss CO art. 957-964).
journal-entries-title = Journal entries
journal-entries-new = New entry
journal-entries-empty-list = No entries yet
journal-entries-col-number = No.
journal-entries-col-date = Date
journal-entries-col-journal = Journal
journal-entries-col-description = Description
journal-entries-col-total = Total
journal-entry-form-title = New journal entry
journal-entry-form-date = Date
journal-entry-form-journal = Journal
journal-entry-form-description = Description
journal-entry-form-add-line = + Add line
journal-entry-form-remove-line = Remove line
journal-entry-form-col-account = Account
journal-entry-form-col-debit = Debit
journal-entry-form-col-credit = Credit
journal-entry-form-total-debit = Total debits
journal-entry-form-total-credit = Total credits
journal-entry-form-diff = Difference
journal-entry-form-balanced = Balanced
journal-entry-form-unbalanced = Unbalanced
journal-entry-form-submit = Save
journal-entry-form-cancel = Cancel
journal-entry-form-incomplete-line = Incomplete line
journal-entry-form-max-decimals = Maximum 4 decimals
journal-entry-form-amount-too-large = Amount too large
account-autocomplete-unavailable = Autocomplete unavailable — enter account ID
journal-achats = Purchases
journal-ventes = Sales
journal-banque = Bank
journal-caisse = Cash
journal-od = Miscellaneous
journal-entry-saved = Entry saved
error-fiscal-year-closed-generic = The fiscal year is closed — no entries can be added or modified (Swiss CO art. 957-964).
error-inactive-accounts = One or more accounts are archived or invalid.

# Edit & delete journal entries (Story 3.3)
journal-entry-edit = Edit
journal-entry-delete = Delete
journal-entry-delete-confirm-title = Delete entry No.{ $number }?
journal-entry-delete-confirm-message = This action is irreversible. The action will be recorded in the audit log.
journal-entry-delete-confirm-cancel = Cancel
journal-entry-delete-confirm-delete = Delete
journal-entry-deleted = Entry deleted
journal-entry-conflict-title = Version conflict
journal-entry-conflict-message = This entry has been modified by another user. Reload?
journal-entry-conflict-reload = Reload
journal-entry-conflict-reloaded = List reloaded — click Edit again to resume
error-date-outside-fiscal-year = The date { $date } is not within the current fiscal year of this entry
error-date-outside-fiscal-year-generic = The date is not within the current fiscal year of this entry

# Search, pagination, sorting (Story 3.4)
journal-entries-filter-description = Description
journal-entries-filter-amount-min = Amount min
journal-entries-filter-amount-max = Amount max
journal-entries-filter-date-from = From date
journal-entries-filter-date-to = To date
journal-entries-filter-journal = Journal
journal-entries-filter-journal-all = All
journal-entries-filter-reset = Reset
journal-entries-pagination-on = of
journal-entries-pagination-prev = Previous
journal-entries-pagination-next = Next
journal-entries-pagination-page-size = Per page
journal-entries-sort-asc-indicator = sorted ascending
journal-entries-sort-desc-indicator = sorted descending
journal-entries-loading = Loading…

# Bilingual tooltips for accounting terms (Story 3.5)
tooltip-debit-natural = Money comes into this account
tooltip-debit-technical = Debit — left column
tooltip-credit-natural = Money goes out from this account
tooltip-credit-technical = Credit — right column
tooltip-journal-natural = Register where similar entries are grouped
tooltip-journal-technical = Accounting journal (Purchases, Sales, Bank, Cash, Miscellaneous)
tooltip-balanced-natural = Total money in equals total money out
tooltip-balanced-technical = Double-entry balanced (debits = credits)

# Story 4.1 — Address book (contacts CRUD)
nav-contacts = Address book
contacts-page-title = Address book
contact-form-create-title = New contact
contact-form-edit-title = Edit contact
contact-form-name = Name / Company name
contact-form-type = Type
contact-form-is-client = Client
contact-form-is-supplier = Supplier
contact-form-email = Email
contact-form-phone = Phone
contact-form-address = Address
contact-form-ide = UID number (CHE)
contact-form-ide-help = Format: CHE-123.456.789
contact-type-personne = Person
contact-type-entreprise = Company
contact-form-submit-create = Create
contact-form-submit-edit = Save
contact-form-cancel = Cancel
contact-list-new = New contact
contact-list-edit = Edit
contact-list-archive = Archive
contact-archive-confirm = Archive
contact-archive-cancel = Cancel
contact-col-name = Name
contact-col-type = Type
contact-col-flags = Roles
contact-col-ide = UID
contact-col-email = Email
contact-col-actions = Actions
contact-filter-search-placeholder = Search by name or email…
contact-filter-type-all = All types
contact-filter-archived = Include archived
contact-empty-list = No contacts. Create your first contact with the "New contact" button.
contact-created-success = Contact created
contact-updated-success = Contact updated
contact-archived-success = Contact archived
contact-archive-confirm-title = Archive contact?
contact-archive-confirm-body = The contact will no longer appear in the default list. You can still view it by enabling "Include archived".
contact-error-name-required = Name is required
contact-error-name-too-long = Name must be at most 255 characters
contact-error-email-invalid = Invalid email format
contact-error-ide-invalid = Invalid Swiss UID number (format or checksum)
contact-error-ide-duplicate = A contact with this UID number already exists
contact-error-not-found = Contact not found
contact-error-archived-no-modify = Contact archived — no further modification or archiving allowed
contact-conflict-title = Version conflict
contact-conflict-body = This contact has been modified elsewhere. Reload the current version?
error-ide-already-exists = A contact with this UID number already exists

# Story 4.2 — Payment terms & product catalog
contact-form-payment-terms = Payment terms
contact-form-payment-terms-placeholder = e.g. 30 days net
products-page-title = Product / service catalog
product-form-create-title = New product
product-form-edit-title = Edit product
product-form-name = Name
product-form-description = Description
product-form-price = Unit price
product-form-vat-rate = VAT rate
product-form-vat-help = Swiss VAT rates effective from 01.01.2024
product-vat-exempt = 0.00% — Exempt
product-vat-reduced = 2.60% — Reduced rate
product-vat-special = 3.80% — Accommodation
product-vat-normal = 8.10% — Standard rate
product-list-new = New product
product-list-edit = Edit
product-list-archive = Archive
product-col-name = Name
product-col-description = Description
product-col-price = Price
product-col-vat = VAT
product-col-actions = Actions
product-filter-search = Search by name or description…
product-filter-archived = Include archived
product-empty-list = No products. Create your first product with the "New product" button.
product-created-success = Product created
product-updated-success = Product updated
product-archived-success = Product archived
product-error-name-required = Name is required
product-error-name-too-long = Name must be at most 255 characters
product-error-price-required = Price is required
product-error-price-negative = Price must be positive or zero
product-error-price-invalid = Invalid price format
product-error-vat-invalid = VAT rate not allowed
product-error-name-duplicate = A product with this name already exists
product-archive-confirm-title = Archive this product?
product-archive-confirm-body = The product will no longer be visible in the default list. You can still view it by enabling "Include archived".
product-conflict-title = Version conflict
product-conflict-body = This product was modified elsewhere. Do you want to reload the current version?
product-filter-reset = Reset
product-pagination-prev = Previous
product-pagination-next = Next
product-pagination-of = of
product-conflict-reload = Reload
product-form-cancel = Cancel
product-form-submit-create = Create
product-form-submit-edit = Save
product-archive-cancel = Cancel
product-archive-confirm = Archive

# --- Story 5.1: Draft invoices ---
nav-invoices = Invoices
invoices-page-title = Invoices
invoice-new-title = New invoice
invoice-edit-title = Edit invoice
invoice-view-title = Invoice
invoice-form-contact = Contact
invoice-form-date = Date
invoice-form-due-date = Due date
invoice-form-payment-terms = Payment terms
invoice-form-status = Status
invoice-form-number = Invoice no.
invoice-line-description = Description
invoice-line-quantity = Quantity
invoice-line-unit-price = Unit price
invoice-line-vat-rate = VAT %
invoice-line-total = Total
invoice-line-actions = Actions
invoice-add-free-line = Free line
invoice-add-from-catalog = From catalog
invoice-col-date = Date
invoice-col-contact = Contact
invoice-col-number = No.
invoice-col-status = Status
invoice-col-total = Total
invoice-col-actions = Actions
invoice-status-draft = Draft
invoice-status-validated = Validated
invoice-status-cancelled = Cancelled
invoice-filter-search = Search…
invoice-filter-status-all = All statuses
invoice-filter-contact-all = All contacts
invoice-filter-date-from = From
invoice-filter-date-to = To
invoice-new-button = New invoice
invoice-edit-button = Edit
invoice-delete-button = Delete
invoice-subtotal = Subtotal
invoice-total = Total
invoice-empty-list = No invoices. Create your first one via "New invoice".
invoice-created-success = Invoice created
invoice-updated-success = Invoice updated
invoice-deleted-success = Invoice deleted
invoice-delete-confirm-title = Delete invoice?
invoice-delete-confirm-body = This draft invoice will be permanently deleted.
invoice-conflict-title = Version conflict
invoice-conflict-body = This invoice was modified elsewhere. Reload current version?
invoice-error-no-lines = An invoice must contain at least one line
invoice-error-contact-required = Please select a contact
invoice-error-contact-invalid = Contact not found
invoice-error-quantity-positive = Quantity must be strictly positive
invoice-error-description-required = Description is required
invoice-error-vat-invalid = VAT rate not allowed. Accepted: 0.00%, 2.60%, 3.80%, 8.10%
invoice-error-illegal-state = This invoice can no longer be edited
invoice-product-picker-title = Select a product
invoice-product-picker-search = Search product…
invoice-product-picker-empty = No products
invoice-contact-picker-placeholder = Search contact…
invoice-contact-picker-empty = No contacts

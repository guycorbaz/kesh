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

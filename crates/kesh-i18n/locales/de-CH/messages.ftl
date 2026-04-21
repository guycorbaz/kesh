# Kesh — Nachrichten Deutsch (Schweiz)

# Authentifizierungsfehler
error-invalid-credentials = Ungültige Anmeldedaten
error-unauthenticated = Nicht authentifiziert
error-invalid-refresh-token = Sitzung abgelaufen
error-rate-limited = Zu viele Versuche

# Autorisierungsfehler
error-forbidden = Zugriff verweigert
error-cannot-disable-self = Das eigene Konto kann nicht deaktiviert werden
error-cannot-disable-last-admin = Der letzte Administrator kann nicht deaktiviert werden

# Ressourcenfehler
error-not-found = Ressource nicht gefunden
error-conflict = Ressource bereits vorhanden
error-optimistic-lock = Versionskonflikt — die Ressource wurde geändert
error-foreign-key = Ungültige Referenz
error-check-constraint = Ungültiger Wert
error-illegal-state = Unzulässiger Statusübergang

# Validierungsfehler
error-validation = Validierungsfehler
error-username-empty = Der Benutzername darf nicht leer sein
error-username-too-long = Der Benutzername darf nicht länger als { $max } Zeichen sein

# Systemfehler
error-internal = Interner Fehler
error-service-unavailable = Dienst vorübergehend nicht verfügbar

# Onboarding-Fehler (Story 2.2)
error-onboarding-step-already-completed = Dieser Konfigurationsschritt wurde bereits abgeschlossen

# Onboarding — Assistent
onboarding-choose-mode = Wählen Sie Ihren Nutzungsmodus
onboarding-mode-guided = Geführt
onboarding-mode-guided-desc = Grosszügige Abstände, kontextuelle Hilfe, Bestätigungen vor Aktionen
onboarding-mode-expert = Experte
onboarding-mode-expert-desc = Kompakte Oberfläche, Tastenkürzel, direkte Aktionen
onboarding-choose-path = Wie möchten Sie beginnen?
onboarding-path-demo = Mit Demodaten erkunden
onboarding-path-demo-desc = Entdecken Sie Kesh mit realistischen Beispieldaten
onboarding-path-production = Für die Produktion konfigurieren
onboarding-path-production-desc = Konfigurieren Sie Ihre Organisation, um loszulegen

# Demo-Banner
demo-banner-text = Demonstrationsinstanz — fiktive Daten
demo-banner-reset = Für die Produktion zurücksetzen
demo-reset-confirm-title = Instanz zurücksetzen
demo-reset-confirm-body = Alle Demonstrationsdaten werden gelöscht. Möchten Sie fortfahren?
demo-reset-confirm-ok = Bestätigen
demo-reset-confirm-cancel = Abbrechen

# Navigation sidebar (Story 6.3)
nav-home = Startseite
nav-contacts = Kontakte
nav-products = Katalog
nav-invoices = Rechnungen
nav-invoicing-due-dates = Fälligkeiten
nav-settings = Einstellungen

# Onboarding — Pfad B (Story 2.3)
onboarding-choose-org-type = Organisationstyp
onboarding-org-independant = Selbstständig
onboarding-org-independant-desc = Freiberufler, selbstständig Erwerbender
onboarding-org-association = Verein
onboarding-org-association-desc = Gemeinnütziger Verein
onboarding-org-pme = KMU
onboarding-org-pme-desc = Klein- und mittelständisches Unternehmen (AG, GmbH)
onboarding-choose-accounting-lang = Buchhaltungssprache
onboarding-accounting-lang-desc = Sprache der Kontenplan-Bezeichnungen (unabhängig von der Oberflächensprache)
onboarding-coordinates-title = Angaben zu Ihrer Organisation
onboarding-field-name = Name / Firma
onboarding-field-address = Adresse
onboarding-field-ide = UID-Nummer
onboarding-field-ide-hint = optional, Format CHE-xxx.xxx.xxx
onboarding-bank-title = Hauptbankkonto
onboarding-field-bank-name = Bankname
onboarding-field-iban = IBAN
onboarding-field-qr-iban = QR-IBAN
onboarding-skip-bank = Später konfigurieren
onboarding-next = Weiter
incomplete-banner-text = Konfiguration unvollständig — Einrichtung abschliessen
incomplete-banner-cta = Einrichtung abschliessen

# Startseite (Story 2.4)
homepage-title = Übersicht
homepage-entries-title = Letzte Buchungen
homepage-entries-empty = Keine Buchungen.
homepage-entries-empty-guided = Noch keine Buchungen. Erfassen Sie Ihre erste Buchung.
homepage-entries-action = Buchung erfassen
homepage-invoices-title = Offene Rechnungen
homepage-invoices-empty = Keine offenen Rechnungen.
homepage-invoices-empty-guided = Keine offenen Rechnungen. Erstellen Sie Ihre erste Rechnung.
homepage-invoices-action = Rechnung erstellen
homepage-bank-title = Bankkonten
homepage-bank-empty = Kein Bankkonto.
homepage-bank-empty-guided = Kein Bankkonto konfiguriert. Fügen Sie Ihr Konto hinzu, um Kontoauszüge zu importieren.
homepage-bank-no-transactions = Keine importierten Transaktionen
homepage-bank-action = Konfigurieren

# Einstellungen (Story 2.4)
settings-title = Einstellungen
settings-org-title = Organisation
settings-accounting-title = Buchhaltung
settings-bank-title = Bankkonten
settings-users-title = Benutzer
settings-field-name = Name
settings-field-address = Adresse
settings-field-ide = UID
settings-field-org-type = Organisationstyp
settings-field-instance-language = Oberflächensprache
settings-field-accounting-language = Buchhaltungssprache
search-coming-soon = Suche bald verfügbar

# Misc i18n (Story 2.4 review)
loading = Laden...
settings-edit = Bearbeiten
settings-edit-coming-soon = Bearbeitung bald verfügbar
settings-manage = Verwalten
settings-no-bank = Kein Bankkonto konfiguriert.
settings-no-company = Keine Organisation konfiguriert. Schliessen Sie das Onboarding ab.

# Kontenplan (Story 3.1)
accounts-title = Kontenplan
accounts-add = Neues Konto
accounts-edit = Konto bearbeiten
accounts-archive = Archivieren
accounts-archive-confirm = Das Konto wird in zukünftigen Auswahlen nicht mehr verfügbar sein, bleibt aber in bestehenden Buchungen sichtbar.
account-field-number = Nummer
account-field-name = Name
account-field-type = Typ
account-field-parent = Übergeordnetes Konto
account-type-asset = Aktiv
account-type-liability = Passiv
account-type-revenue = Ertrag
account-type-expense = Aufwand
account-archived-label = Archiviert

# Mode Geführt/Experte (Story 2.5)
mode-guided-label = Geführt
mode-expert-label = Experte
shortcut-new-entry = Ctrl+N : Neue Buchung

# Buchungen (Story 3.2)
error-entry-unbalanced = Unausgeglichene Buchung — die Summe der Soll-Beträge ({ $debit }) entspricht nicht der Summe der Haben-Beträge ({ $credit })
error-no-fiscal-year = Kein Geschäftsjahr existiert für das Datum { $date }. Erstellen Sie ein Geschäftsjahr, bevor Sie Buchungen erfassen.
error-fiscal-year-closed = Das Geschäftsjahr für das Datum { $date } ist abgeschlossen — keine Buchungen können hinzugefügt oder geändert werden (OR Art. 957-964).
journal-entries-title = Buchungen
journal-entries-new = Neue Buchung
journal-entries-empty-list = Noch keine Buchungen erfasst
journal-entries-col-number = Nr.
journal-entries-col-date = Datum
journal-entries-col-journal = Journal
journal-entries-col-description = Beschreibung
journal-entries-col-total = Betrag
journal-entry-form-title = Buchungserfassung
journal-entry-form-date = Datum
journal-entry-form-journal = Journal
journal-entry-form-description = Beschreibung
journal-entry-form-add-line = + Zeile hinzufügen
journal-entry-form-remove-line = Zeile entfernen
journal-entry-form-col-account = Konto
journal-entry-form-col-debit = Soll
journal-entry-form-col-credit = Haben
journal-entry-form-total-debit = Summe Soll
journal-entry-form-total-credit = Summe Haben
journal-entry-form-diff = Differenz
journal-entry-form-balanced = Ausgeglichen
journal-entry-form-unbalanced = Unausgeglichen
journal-entry-form-submit = Speichern
journal-entry-form-cancel = Abbrechen
journal-entry-form-incomplete-line = Unvollständige Zeile
journal-entry-form-max-decimals = Maximal 4 Nachkommastellen
journal-entry-form-amount-too-large = Betrag zu hoch
account-autocomplete-unavailable = Autovervollständigung nicht verfügbar — Konto-ID eingeben
journal-achats = Einkäufe
journal-ventes = Verkäufe
journal-banque = Bank
journal-caisse = Kasse
journal-od = Diverse Operationen
journal-entry-saved = Buchung gespeichert
error-fiscal-year-closed-generic = Das Geschäftsjahr ist abgeschlossen — keine Buchungen können hinzugefügt oder geändert werden (OR Art. 957-964).
error-inactive-accounts = Ein oder mehrere Konten sind archiviert oder ungültig.

# Buchung bearbeiten & löschen (Story 3.3)
journal-entry-edit = Bearbeiten
journal-entry-delete = Löschen
journal-entry-delete-confirm-title = Buchung Nr.{ $number } löschen?
journal-entry-delete-confirm-message = Diese Aktion ist unwiderruflich. Die Aktion wird im Prüfprotokoll gespeichert.
journal-entry-delete-confirm-cancel = Abbrechen
journal-entry-delete-confirm-delete = Löschen
journal-entry-deleted = Buchung gelöscht
journal-entry-conflict-title = Versionskonflikt
journal-entry-conflict-message = Diese Buchung wurde von einem anderen Benutzer geändert. Neu laden?
journal-entry-conflict-reload = Neu laden
journal-entry-conflict-reloaded = Liste aktualisiert — klicken Sie erneut auf Bearbeiten
error-date-outside-fiscal-year = Das Datum { $date } liegt nicht im aktuellen Geschäftsjahr dieser Buchung
error-date-outside-fiscal-year-generic = Das Datum liegt nicht im aktuellen Geschäftsjahr dieser Buchung

# Suche, Paginierung, Sortierung (Story 3.4)
journal-entries-filter-description = Beschreibung
journal-entries-filter-amount-min = Betrag min
journal-entries-filter-amount-max = Betrag max
journal-entries-filter-date-from = Von Datum
journal-entries-filter-date-to = Bis Datum
journal-entries-filter-journal = Journal
journal-entries-filter-journal-all = Alle
journal-entries-filter-reset = Zurücksetzen
journal-entries-pagination-on = von
journal-entries-pagination-prev = Zurück
journal-entries-pagination-next = Weiter
journal-entries-pagination-page-size = Pro Seite
journal-entries-sort-asc-indicator = aufsteigend sortiert
journal-entries-sort-desc-indicator = absteigend sortiert
journal-entries-loading = Wird geladen…

# Zweisprachige Tooltips Buchhaltungsbegriffe (Story 3.5)
tooltip-debit-natural = Geld kommt auf dieses Konto
tooltip-debit-technical = Soll — linke Spalte
tooltip-credit-natural = Geld geht von diesem Konto ab
tooltip-credit-technical = Haben — rechte Spalte
tooltip-journal-natural = Register, in dem ähnliche Buchungen gruppiert sind
tooltip-journal-technical = Buchhaltungsjournal (Einkäufe, Verkäufe, Bank, Kasse, Diverse)
tooltip-balanced-natural = Die Summe der Eingänge entspricht der Summe der Ausgänge
tooltip-balanced-technical = Doppelte Buchführung im Gleichgewicht (Soll = Haben)

# Story 4.1 — Adressbuch (Kontakte CRUD)
contacts-page-title = Adressbuch
contact-form-create-title = Neuer Kontakt
contact-form-edit-title = Kontakt bearbeiten
contact-form-name = Name / Firmenname
contact-form-type = Typ
contact-form-is-client = Kunde
contact-form-is-supplier = Lieferant
contact-form-email = E-Mail
contact-form-phone = Telefon
contact-form-address = Adresse
contact-form-ide = UID-Nummer (CHE)
contact-form-ide-help = Format: CHE-123.456.789
contact-type-personne = Person
contact-type-entreprise = Unternehmen
contact-form-submit-create = Erstellen
contact-form-submit-edit = Speichern
contact-form-cancel = Abbrechen
contact-list-new = Neuer Kontakt
contact-list-edit = Bearbeiten
contact-list-archive = Archivieren
contact-archive-confirm = Archivieren
contact-archive-cancel = Abbrechen
contact-col-name = Name
contact-col-type = Typ
contact-col-flags = Rollen
contact-col-ide = UID
contact-col-email = E-Mail
contact-col-actions = Aktionen
contact-filter-search-placeholder = Nach Name oder E-Mail suchen…
contact-filter-type-all = Alle Typen
contact-filter-archived = Archivierte einschliessen
contact-empty-list = Keine Kontakte. Erstellen Sie Ihren ersten Kontakt mit der Schaltfläche „Neuer Kontakt".
contact-created-success = Kontakt erstellt
contact-updated-success = Kontakt aktualisiert
contact-archived-success = Kontakt archiviert
contact-archive-confirm-title = Kontakt archivieren?
contact-archive-confirm-body = Der Kontakt wird standardmässig nicht mehr angezeigt. Sie können ihn weiterhin über „Archivierte einschliessen" einsehen.
contact-error-name-required = Der Name ist erforderlich
contact-error-name-too-long = Der Name darf höchstens 255 Zeichen enthalten
contact-error-email-invalid = Ungültiges E-Mail-Format
contact-error-ide-invalid = Ungültige schweizerische UID-Nummer (Format oder Prüfsumme)
contact-error-ide-duplicate = Ein Kontakt mit dieser UID-Nummer existiert bereits
contact-error-not-found = Kontakt nicht gefunden
contact-error-archived-no-modify = Kontakt archiviert — Änderung oder weitere Archivierung nicht erlaubt
contact-conflict-title = Versionskonflikt
contact-conflict-body = Dieser Kontakt wurde anderswo geändert. Möchten Sie die aktuelle Version neu laden?
error-ide-already-exists = Ein Kontakt mit dieser UID-Nummer existiert bereits

# Story 4.2 — Zahlungsbedingungen & Produktkatalog
contact-form-payment-terms = Zahlungsbedingungen
contact-form-payment-terms-placeholder = z. B. 30 Tage netto
products-page-title = Produkt-/Dienstleistungskatalog
product-form-create-title = Neues Produkt
product-form-edit-title = Produkt bearbeiten
product-form-name = Name
product-form-description = Beschreibung
product-form-price = Einzelpreis
product-form-vat-rate = MWST-Satz
product-form-vat-help = In der Schweiz seit 01.01.2024 gültige Sätze
product-vat-exempt = 0.00 % — Befreit
product-vat-reduced = 2.60 % — Reduzierter Satz
product-vat-special = 3.80 % — Beherbergung
product-vat-normal = 8.10 % — Normalsatz
product-list-new = Neues Produkt
product-list-edit = Bearbeiten
product-list-archive = Archivieren
product-col-name = Name
product-col-description = Beschreibung
product-col-price = Preis
product-col-vat = MWST
product-col-actions = Aktionen
product-filter-search = Nach Name oder Beschreibung suchen…
product-filter-archived = Archivierte einschliessen
product-empty-list = Keine Produkte. Erstellen Sie Ihr erstes Produkt mit « Neues Produkt ».
product-created-success = Produkt erstellt
product-updated-success = Produkt geändert
product-archived-success = Produkt archiviert
product-error-name-required = Der Name ist erforderlich
product-error-name-too-long = Der Name darf höchstens 255 Zeichen lang sein
product-error-price-required = Preis ist erforderlich
product-error-price-negative = Der Preis muss positiv oder null sein
product-error-price-invalid = Ungültiges Preisformat
product-error-vat-invalid = MWST-Satz nicht erlaubt
product-error-name-duplicate = Ein Produkt mit diesem Namen existiert bereits
product-archive-confirm-title = Produkt archivieren?
product-archive-confirm-body = Das Produkt wird in der Standardliste nicht mehr angezeigt. Sie können es weiterhin einsehen, indem Sie « Archivierte einschliessen » aktivieren.
product-conflict-title = Versionskonflikt
product-conflict-body = Dieses Produkt wurde anderweitig geändert. Möchten Sie die aktuelle Version neu laden?
product-filter-reset = Zurücksetzen
product-pagination-prev = Zurück
product-pagination-next = Weiter
product-pagination-of = von
product-conflict-reload = Neu laden
product-form-cancel = Abbrechen
product-form-submit-create = Erstellen
product-form-submit-edit = Speichern
product-archive-cancel = Abbrechen
product-archive-confirm = Archivieren

# --- Story 5.1: Rechnungsentwürfe ---
invoices-page-title = Rechnungen
invoice-new-title = Neue Rechnung
invoice-edit-title = Rechnung bearbeiten
invoice-view-title = Rechnung
invoice-form-contact = Kontakt
invoice-form-date = Datum
invoice-form-due-date = Fälligkeit
invoice-form-payment-terms = Zahlungsbedingungen
invoice-form-status = Status
invoice-form-number = Rechnungsnr.
invoice-line-description = Beschreibung
invoice-line-quantity = Menge
invoice-line-unit-price = Einzelpreis
invoice-line-vat-rate = MwSt %
invoice-line-total = Total
invoice-line-actions = Aktionen
invoice-add-free-line = Freie Zeile
invoice-add-from-catalog = Aus Katalog
invoice-col-date = Datum
invoice-col-contact = Kontakt
invoice-col-number = Nr.
invoice-col-status = Status
invoice-col-total = Total
invoice-col-actions = Aktionen
invoice-status-draft = Entwurf
invoice-status-validated = Validiert
invoice-status-cancelled = Storniert
invoice-filter-search = Suchen…
invoice-filter-status-all = Alle Status
invoice-filter-contact-all = Alle Kontakte
invoice-filter-date-from = Von
invoice-filter-date-to = Bis
invoice-new-button = Neue Rechnung
invoice-edit-button = Bearbeiten
invoice-delete-button = Löschen
invoice-subtotal = Zwischensumme
invoice-total = Total
invoice-empty-list = Keine Rechnungen. Erstellen Sie Ihre erste Rechnung mit «Neue Rechnung».
invoice-created-success = Rechnung erstellt
invoice-updated-success = Rechnung aktualisiert
invoice-deleted-success = Rechnung gelöscht
invoice-delete-confirm-title = Rechnung löschen?
invoice-delete-confirm-body = Diese Entwurfsrechnung wird endgültig gelöscht.
invoice-conflict-title = Versionskonflikt
invoice-conflict-body = Diese Rechnung wurde andernorts geändert. Aktuelle Version laden?
invoice-error-no-lines = Eine Rechnung muss mindestens eine Zeile enthalten
invoice-error-contact-required = Bitte wählen Sie einen Kontakt
invoice-error-contact-invalid = Kontakt nicht gefunden
invoice-error-quantity-positive = Die Menge muss streng positiv sein
invoice-error-description-required = Die Beschreibung ist obligatorisch
invoice-error-vat-invalid = MwSt-Satz nicht erlaubt. Akzeptiert: 0.00%, 2.60%, 3.80%, 8.10%
invoice-error-illegal-state = Diese Rechnung kann nicht mehr geändert werden
invoice-product-picker-title = Produkt auswählen
invoice-product-picker-search = Produkt suchen…
invoice-product-picker-empty = Keine Produkte
invoice-contact-picker-placeholder = Kontakt suchen…
invoice-contact-picker-empty = Keine Kontakte

# Story 5.2 — Validation & numérotation (TODO: traduction — fallback fr-CH via kesh-i18n)

# --- Story 5.3 — PDF QR-Rechnung ---

invoice-pdf-title = Rechnung
invoice-pdf-date = Datum
invoice-pdf-due-date = Fälligkeit
invoice-pdf-number = Rechnungsnummer
invoice-pdf-ide = UID
invoice-pdf-recipient = Empfänger
invoice-pdf-description = Beschreibung
invoice-pdf-quantity = Menge
invoice-pdf-unit-price = Einzelpreis
invoice-pdf-vat = MWST
invoice-pdf-line-total = Total
invoice-pdf-subtotal = Zwischentotal
invoice-pdf-total = Total
invoice-pdf-total-ttc = Total inkl. MWST
invoice-pdf-payment-terms = Zahlungsbedingungen
invoice-pdf-qr-section-payment = Zahlteil
invoice-pdf-qr-section-receipt = Empfangsschein
invoice-pdf-qr-account = Konto / Zahlbar an
invoice-pdf-qr-reference = Referenz
invoice-pdf-qr-additional-info = Zusätzliche Informationen
invoice-pdf-qr-payable-by = Zahlbar durch
invoice-pdf-qr-currency = Währung
invoice-pdf-qr-amount = Betrag
invoice-pdf-qr-acceptance-point = Annahmestelle
invoice-pdf-qr-separate-before-paying = Vor der Einzahlung abzutrennen

invoice-pdf-error-invoice-not-validated = Die Rechnung muss validiert sein, bevor ein PDF erzeugt werden kann.
invoice-pdf-error-invoice-not-pdf-ready = Die Rechnung ist nicht bereit für die PDF-Erzeugung.
invoice-pdf-error-pdf-generation-failed = PDF-Erzeugung fehlgeschlagen. Bitte später erneut versuchen.
invoice-pdf-error-popup-blocked = Pop-up vom Browser blockiert — bitte Pop-ups für das PDF erlauben.
invoice-pdf-error-missing-contact-address = Kundenadresse fehlt — bitte in der Kontaktkarte erfassen.
invoice-pdf-error-missing-primary-bank-account = Kein Hauptbankkonto konfiguriert — bitte in den Einstellungen hinzufügen.

invoices-download-pdf = PDF herunterladen
invoices-download-pdf-aria-label = Rechnung { $number } als PDF herunterladen

error-invoice-not-validated = Die Rechnung muss validiert sein, bevor ein PDF erzeugt werden kann.
error-invoice-too-many-lines-for-pdf = Die Rechnung enthält { $count } Positionen — das A4-PDF ist in v0.1 auf { $max } Positionen begrenzt.
error-pdf-generation-failed = PDF-Erzeugung fehlgeschlagen.

# Story 5.4 — Fälligkeitsplan Rechnungen
due-dates-title = Fälligkeitsplan
due-dates-filter-all = Alle
due-dates-filter-unpaid = Unbezahlt
due-dates-filter-overdue = Überfällig
due-dates-filter-paid = Bezahlt
due-dates-summary-unpaid = unbezahlte Rechnungen
due-dates-summary-overdue = überfällig
due-dates-search-label = Suche
due-dates-contact-label = Kontakt
due-dates-contact-placeholder = Alle Kontakte
due-dates-due-before-label = Fällig vor
due-dates-column-date = Datum
due-dates-column-due-date = Fälligkeit
due-dates-column-contact = Kunde
due-dates-column-total = Total
due-dates-column-payment-status = Status
due-dates-column-paid-at = Bezahlt am
due-dates-export-button = CSV exportieren
due-dates-no-results = Keine Rechnungen anzuzeigen.
due-dates-result-suffix = Ergebnis(se)

payment-status-paid = Bezahlt
payment-status-unpaid = Unbezahlt
payment-status-overdue = Überfällig

invoice-mark-paid-button = Als bezahlt markieren
invoice-mark-paid-dialog-title = Rechnung als bezahlt markieren
invoice-mark-paid-dialog-body = Geben Sie das Datum an, an dem Sie die Zahlung erhalten haben.
invoice-mark-paid-date-label = Zahlungsdatum
invoice-mark-paid-confirm = Zahlung bestätigen
invoice-mark-paid-success = Rechnung als bezahlt markiert
invoice-unmark-paid-button = Zahlung rückgängig machen
invoice-unmark-paid-dialog-title = Zahlung rückgängig machen
invoice-unmark-paid-dialog-body = Die Rechnung gilt wieder als unbezahlt. Nützlich zur Korrektur eines Fehlers. Fortfahren?
invoice-unmark-paid-confirm = Rückgängig machen
invoice-unmark-paid-success = Zahlung rückgängig gemacht
invoice-detail-paid-at-label = Bezahlt am

invoice-error-paid-at-required = Zahlungsdatum erforderlich
invoice-error-paid-at-before-invoice-date = Das Zahlungsdatum darf nicht vor dem Rechnungsdatum liegen
invoice-error-mark-paid-not-validated = Nur validierte Rechnungen können als bezahlt markiert werden
invoice-error-already-unpaid = Diese Rechnung ist nicht als bezahlt markiert

echeancier-csv-header-number = Nummer
echeancier-csv-header-date = Datum
echeancier-csv-header-due-date = Fälligkeitsdatum
echeancier-csv-header-contact = Kunde
echeancier-csv-header-total = Total
echeancier-csv-header-payment-status = Zahlungsstatus
echeancier-csv-header-paid-at = Zahlungsdatum
echeancier-export-error-too-large = Zu viele Ergebnisse (> { $limit }). Bitte die Filter verfeinern (z. B. Datumsbereich oder Zahlungsstatus), bevor der Export erneut gestartet wird.
invoice-pdf-error-contact-missing = Der mit der Rechnung verknüpfte Kontakt wurde nicht gefunden.
invoice-pdf-error-no-primary-bank = Für dieses Unternehmen ist kein Hauptbankkonto konfiguriert.
invoice-pdf-error-company-address-empty = Die Unternehmensadresse ist leer — bitte vor der PDF-Erstellung ausfüllen.
invoice-pdf-error-client-address-required = Die Kundenadresse ist zur PDF-Erstellung erforderlich.
invoice-pdf-error-client-address-empty = Die Kundenadresse ist leer — bitte vor der PDF-Erstellung ausfüllen.

common-loading = Laden…
common-previous = Vorherige
common-next = Nächste
common-cancel = Abbrechen
common-error = Unerwarteter Fehler

invoice-pdf-error-not-found = Rechnung nicht gefunden.
invoice-pdf-error-generic = Fehler beim Herunterladen des PDFs.
invoice-pdf-error-empty = Das empfangene PDF ist leer.

# Story 2.6 — Onboarding: Invoice Settings Pre-fill
config-incomplete-title = Konfiguration unvollständig
config-incomplete-link = Konfigurieren Sie die Abrechnungskonten
invoice-settings-required = Konfigurieren Sie zunächst die Abrechnungskonten in den Einstellungen

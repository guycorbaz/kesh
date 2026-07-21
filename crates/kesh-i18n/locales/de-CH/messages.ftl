# Kesh — Nachrichten Deutsch (Schweiz)

# Authentifizierungsfehler
error-invalid-credentials = Ungültige Anmeldedaten
error-unauthenticated = Nicht authentifiziert
error-invalid-refresh-token = Sitzung abgelaufen
error-rate-limited = Zu viele Versuche

# Autorisierungsfehler
error-forbidden = Zugriff verweigert
error-api-key-read-only = Dieser API-Schlüssel ist schreibgeschützt (Scope «read»). Nur GET-Anfragen sind erlaubt.
error-api-key-management-forbidden = Die Verwaltung von API-Schlüsseln ist über einen API-Schlüssel nicht erlaubt. Verwenden Sie die Weboberfläche.
error-cannot-disable-self = Das eigene Konto kann nicht deaktiviert werden
error-cannot-disable-last-admin = Der letzte Administrator kann nicht deaktiviert werden

# Ressourcenfehler
error-not-found = Ressource nicht gefunden
error-conflict = Ressource bereits vorhanden
error-optimistic-lock = Versionskonflikt — die Ressource wurde geändert
error-foreign-key = Ungültige Referenz
error-journal-entry-linked-to-invoice = Dieser Buchungssatz wurde durch eine validierte Rechnung erzeugt und kann nicht direkt gelöscht werden. Stornieren Sie zuerst die betreffende Rechnung.
error-check-constraint = Ungültiger Wert
error-illegal-state = Unzulässiger Statusübergang

# Validierungsfehler
error-validation = Validierungsfehler
error-email-invalid = Ungültiges E-Mail-Format
error-username-empty = Der Benutzername darf nicht leer sein
error-username-too-long = Der Benutzername darf nicht länger als { $max } Zeichen sein
error-username-contains-at = Der Benutzername darf das Zeichen „@“ nicht enthalten
error-email-template-unknown-variables = Die Vorlage enthält unbekannte Variablen

# Systemfehler
error-internal = Interner Fehler
error-service-unavailable = Dienst vorübergehend nicht verfügbar
db-unavailable-banner = Datenbank vorübergehend nicht verfügbar — automatischer Wiederholungsversuch läuft

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

# Navigation sidebar (Story 6.3 + v014-1)
nav-home = Startseite
nav-contacts = Kontakte
nav-products = Katalog
nav-invoices = Rechnungen
nav-supplier-invoices = Lieferantenrechnungen
nav-payment-batches = Lieferantenzahlungen
nav-invoicing-due-dates = Fälligkeiten
nav-invoicing-reminders = Mahnungen
nav-settings = Einstellungen
# Story v014-1 — restructuration sidebar
nav-quotidien = Täglich
nav-mensuel = Monatlich
nav-administration = Administration
nav-accounts = Kontenrahmen
nav-fiscal-years = Geschäftsjahre
nav-bank-accounts = Bankkonten
nav-bank-profiles = Bankprofile
nav-reconciliation-rules = Zuordnungsregeln

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
onboarding-stub-name-notice = Ihr Unternehmen hat einen vorläufigen Namen — vervollständigen Sie Ihre Angaben
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
homepage-reminders-count = { $n } Rechnung(en) zu mahnen
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

# Assistent Vorsteuer-Einkauf (Story 18-1c)
vat-purchase-title = MWST-Einkaufsassistent
vat-purchase-config-required = Konfigurieren Sie das Vorsteuerkonto unter Einstellungen → Fakturierung, um den Assistenten zu verwenden.
vat-purchase-no-rates = Kein MWST-Satz konfiguriert — siehe Einstellungen → MWST-Sätze.
vat-purchase-charge-account = Aufwandskonto
vat-purchase-ht = Betrag exkl. MWST
vat-purchase-rate = MWST-Satz
vat-purchase-rate-placeholder = Satz wählen
vat-purchase-counterparty = Gegenkonto
vat-purchase-same-account = Aufwandskonto und Gegenkonto müssen unterschiedlich sein.
vat-purchase-recoverable-conflict = Aufwandskonto und Gegenkonto dürfen nicht das Vorsteuerkonto sein.
vat-purchase-insert = Zeilen einfügen
vat-purchase-description = Einkauf — MWST { $rate } % abziehbar
vat-purchase-description-exempt = Einkauf — ohne MWST
vat-purchase-replace-title = Entwurf ersetzen?
vat-purchase-replace-message = Es wurden bereits Zeilen oder ein Text erfasst. Beim Fortfahren wird der aktuelle Entwurf überschrieben.
vat-purchase-replace-confirm = Ersetzen
# Kategorien MWST-Sätze (réutilisées par l'assistant TVA achat, Story 18-1c)
vat-category-normal = Normalsatz
vat-category-reduced = Reduzierter Satz
vat-category-special = Sondersatz (Beherbergung)
vat-category-exempt = Befreit / 0 %
vat-category-custom = Benutzerdefiniert
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
contact-error-payment-terms-days-range = Die Zahlungsfrist muss eine ganze Zahl zwischen 0 und 365 Tagen sein
contact-error-ide-duplicate = Ein Kontakt mit dieser UID-Nummer existiert bereits
contact-error-not-found = Kontakt nicht gefunden
contact-error-archived-no-modify = Kontakt archiviert — Änderung oder weitere Archivierung nicht erlaubt
contact-conflict-title = Versionskonflikt
contact-conflict-body = Dieser Kontakt wurde anderswo geändert. Möchten Sie die aktuelle Version neu laden?
error-ide-already-exists = Ein Kontakt mit dieser UID-Nummer existiert bereits

# Story 4.2 — Zahlungsbedingungen & Produktkatalog
contact-form-payment-terms = Zahlungsbedingungen
contact-form-payment-terms-placeholder = z. B. 30 Tage netto
contact-payment-terms-days-label = { $days ->
    [one] Zahlbar innert { $days } Tag
   *[other] Zahlbar innert { $days } Tagen
}
contact-payment-terms-immediate-label = Zahlbar sofort
contact-form-payment-terms-days = Zahlungsfrist (Tage)
contact-form-payment-terms-days-hint = Das Fälligkeitsdatum der Rechnungen wird vorausberechnet und der Text der Zahlungsbedingungen automatisch erzeugt.
contact-form-payment-terms-disabled-hint = Text wird automatisch aus der Zahlungsfrist erzeugt.
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
product-error-vat-loading = MWST-Sätze werden geladen, bitte warten…
product-error-vat-fetch-failed = MWST-Sätze konnten nicht geladen werden. Überprüfen Sie die Netzwerkverbindung und laden Sie die Seite neu.
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
invoices-settings-vat-accounts-title = MWST-Konten
invoices-settings-vat-accounts-hint = Konten für die MWST-Verbuchung (vorbereitet für die MWST-Abrechnung der ESTV).
invoices-settings-vat-payable = Konto geschuldete MWST (Passiv)
invoices-settings-vat-recoverable = Konto Vorsteuer (Aktiv)
invoices-settings-vat-decompte = Konto MWST-Abrechnung (Passiv)
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
# Story 21-6a (D10) — suspension des rappels : badge + filtre en liste factures.
invoice-paused-badge = Pausiert
invoice-paused-filter-label = Mahnungen
invoice-paused-filter-all = Alle
invoice-paused-filter-paused = Pausiert
invoice-paused-filter-not-paused = Aktiv
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
invoice-pdf-origin-reference = Ref. Originalrechnung
credit-note-pdf-title = Gutschrift
credit-note-pdf-number = Gutschriftnummer
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
error-invoice-too-many-lines-for-pdf = Die Rechnung enthält { $count } Positionen — das einseitige A4-PDF kann sie mit der MwSt-Zusammenfassung nicht alle darstellen. Reduzieren Sie die Positionen oder teilen Sie die Rechnung.
error-pdf-generation-failed = PDF-Erzeugung fehlgeschlagen.
# Story 9-2a + Pass 1 code-review H1 — dedizierter CSV-Variant.
error-csv-generation-failed = CSV-Erzeugung fehlgeschlagen.

# Story 5.4 — Fälligkeitsplan Rechnungen
due-dates-title = Fälligkeitsplan
due-dates-link-aged = Altersstruktur anzeigen
due-dates-link-reminders = Mahnungen anzeigen
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

# === Story 3.7 — Geschäftsjahresverwaltung (DE-CH) ===

fiscal-year-title = Geschäftsjahre
fiscal-year-list-empty = Keine Geschäftsjahre.
fiscal-year-create-button = Neues Geschäftsjahr
fiscal-year-name-label = Name
fiscal-year-start-date-label = Anfangsdatum
fiscal-year-end-date-label = Enddatum
fiscal-year-status-label = Status
fiscal-year-status-open = Offen
fiscal-year-status-closed = Geschlossen
fiscal-year-rename-button = Umbenennen
fiscal-year-close-button = Schliessen
fiscal-year-close-confirmation-title = Geschäftsjahr schliessen?
fiscal-year-close-confirmation-body = Sie sind dabei, das Geschäftsjahr „{ $name }“ zu schliessen. Diese Aktion ist unwiderruflich: keine Buchung, Rechnung oder Zahlung kann mehr in diesem Zeitraum erfasst werden. Bestätigen?
fiscal-year-close-confirmation-action = Endgültig schliessen
fiscal-year-created = Geschäftsjahr erfolgreich erstellt.
fiscal-year-renamed = Geschäftsjahr umbenannt.
fiscal-year-closed = Geschäftsjahr geschlossen.
error-fiscal-year-overlap = Dieses Geschäftsjahr überschneidet sich mit einem bestehenden Jahr.
error-fiscal-year-name-duplicate = Ein Geschäftsjahr mit diesem Namen existiert bereits.
error-fiscal-year-name-empty = Der Name des Geschäftsjahres ist erforderlich.
error-fiscal-year-name-too-long = Der Name des Geschäftsjahrs ist zu lang (max. 50 Zeichen).
error-fiscal-year-dates-invalid = Ungültige Daten — das Enddatum muss strikt nach dem Anfangsdatum liegen.
error-fiscal-year-already-closed = Dieses Geschäftsjahr ist bereits geschlossen.
error-fiscal-year-conflict = Konflikt im Geschäftsjahr (Name oder Anfangsdatum bereits verwendet).
error-fiscal-year-missing = Erstellen Sie zuerst ein Geschäftsjahr unter Einstellungen → Geschäftsjahre.
error-fiscal-year-closed-for-date = Das Geschäftsjahr, das dieses Datum abdeckt, ist geschlossen. Überprüfen Sie das Datum oder Ihre Geschäftsjahre.
go-to-settings = Einstellungen öffnen
settings-fiscal-years-link = Erstellen, umbenennen oder schliessen Sie die Geschäftsjahre Ihres Unternehmens.


# --- Story 8-1b — CAMT.053 Bankimport ---
bank-import-errors-too-large = Datei zu gross. Maximalgrösse: 10 MiB.
bank-import-errors-malformed-xml = XML-Datei ungültig oder abgeschnitten. Bankexport prüfen.
bank-import-errors-unsupported-version = CAMT.053-Version nicht unterstützt. Akzeptierte Versionen: 001.04 und 001.08.
bank-import-errors-missing-field = Ein erforderliches Feld fehlt in der CAMT.053-Datei.
bank-import-errors-invalid-amount = Ein Betrag in der Datei ist ungültig.
bank-import-errors-invalid-date = Ein Datum in der Datei ist ungültig.
bank-import-errors-balance-mismatch = Der Schlusssaldo entspricht nicht der Summe der Transaktionen. Aktivieren Sie «Trotzdem bestätigen», um den Import fortzusetzen.
bank-import-errors-unsupported-currency = Währung nicht unterstützt. In dieser Version wird nur der Schweizer Franken (CHF) akzeptiert.
bank-import-errors-no-matching-statement = Kein Auszug der Datei entspricht dem ausgewählten Bankkonto.
bank-import-errors-duplicate-file = Diese Datei wurde bereits für dieses Unternehmen importiert.
bank-import-errors-bank-account-not-found = Bankkonto nicht gefunden.
bank-import-errors-parse-failed = Die CAMT.053-Datei konnte nicht verarbeitet werden.

bank-import-warnings-balance-mismatch = Schlusssaldo inkonsistent.
bank-import-warnings-unsupported-currency = Währung in v0.1 nicht unterstützt.
bank-import-warnings-ignored-statements = Einige Auszüge der Datei entsprechen nicht dem ausgewählten Konto und werden ignoriert.
# Story 8-3 — Duplikat-Erkennung + partielle Annahme
bank-import-warnings-duplicate-file = Diese Datei wurde bereits importiert.
bank-import-warnings-duplicate-lines-summary = Transaktionen überschneiden sich mit einem früheren Import.
bank-import-warnings-invalid-lines-summary = ungültige Zeilen in der CSV-Datei erkannt.
bank-import-warnings-invalid-lines-truncated = Erste 100 Fehler angezeigt (Limit erreicht).
bank-import-warnings-encoding-mismatch = Das erkannte Encoding weicht vom Profil ab.

bank-import-labels-page-title = CAMT.053-Bankimport
bank-import-labels-bank-account-selector = Ziel-Bankkonto
bank-import-labels-drop-zone = CAMT.053-Datei hier ablegen oder klicken zum Durchsuchen
bank-import-labels-preview-title = Vorschau
bank-import-labels-confirm-import = Import bestätigen
bank-import-labels-cancel = Abbrechen
bank-import-labels-confirm-balance-mismatch = Trotz Saldoabweichung importieren
# Story 8-3 — Bestätigungs-Flags + KF #70
bank-import-labels-confirm-duplicate-file = Trotz bereits importierter Datei importieren
bank-import-labels-confirm-duplicate-lines = Verhalten bei Duplikaten
bank-import-labels-confirm-duplicate-lines-skip = Duplikate ignorieren (Standard)
bank-import-labels-confirm-duplicate-lines-import = Trotzdem importieren
bank-import-labels-confirm-partial-import = Gültige Zeilen trotzdem importieren
bank-import-labels-confirm-encoding-mismatch = Mit erkanntem Encoding importieren
bank-import-labels-bank-profile-selector = CSV-Bankprofil
bank-import-labels-bank-profile-auto-matched = automatisch erkannt
# L6 / M8 (Pass 1 review)
bank-import-labels-bank-profile-auto-detect-placeholder = Automatische Erkennung
bank-import-info-bank-csv-profile-auto-matched = Bankprofil anhand des Dateinamens automatisch erkannt.
bank-import-info-bank-csv-multiple-profile-matches = Mehrere Profile passen zum Dateinamen ; das erste wurde übernommen.
bank-import-errors-no-valid-lines-to-commit = Keine gültige Zeile zum Importieren in der CSV-Datei.
bank-import-labels-list-title = Frühere Importe
bank-import-labels-import-success = Import erfolgreich.
bank-import-labels-empty = Kein Bankimport.

# Story 8-2 — bank-csv + bank-profile keys
bank-import-csv-errors-no-profile-match = Kein Bankprofil entspricht dieser Datei.
bank-import-csv-errors-unsupported-encoding = Datei-Encoding nicht unterstützt (UTF-8 oder ISO-8859-1 erwartet).
bank-import-csv-errors-encoding-mismatch = Erkanntes Encoding weicht vom Profil ab. Bestätigen Sie via confirmEncodingMismatch=true.
bank-import-csv-errors-partial-failure = Einige Zeilen der CSV-Datei konnten nicht verarbeitet werden.
bank-import-csv-errors-profile-invalid = Bankprofil ungültig.
bank-import-csv-errors-profile-duplicate = Ein Profil mit diesem Banknamen existiert bereits.
bank-import-csv-errors-profile-misconfigured = Bankprofil falsch konfiguriert.
bank-import-csv-errors-empty-file = CSV-Datei leer oder keine Datenzeilen.
bank-import-csv-errors-invalid-date = Ungültiges Datum in einer CSV-Zeile.
bank-import-csv-errors-invalid-amount = Ungültiger Betrag in einer CSV-Zeile.
bank-import-csv-errors-ambiguous-debit-credit = Soll und Haben gleichzeitig gefüllt in derselben Zeile.
bank-import-csv-errors-empty-mandatory-field = Pflichtfeld leer.
bank-import-csv-errors-row-too-short = Zeile zu kurz (fehlende Spalten).
bank-import-csv-warnings-profile-auto-matched = Profil automatisch durch Auto-Match angewendet.
bank-import-csv-warnings-multiple-profile-matches = Mehrere Profile passen zu diesem Dateinamen, das neueste wurde verwendet.
bank-import-csv-warnings-encoding-mismatch = Erkanntes Encoding weicht vom Profil ab.
bank-import-errors-unsupported-format = Dateiformat nicht unterstützt (CAMT.053 XML oder CSV erwartet).
bank-import-profile-labels-page-title = Bank-CSV-Profile
bank-import-profile-labels-page-title-new = Neues Bankprofil
bank-import-profile-labels-page-title-edit = Bankprofil bearbeiten
bank-import-profile-labels-bank-name = Bankname
bank-import-profile-labels-filename-pattern = Dateinamen-Muster (Regex)
bank-import-profile-labels-filename-pattern-help = Regex case-sensitive (verwenden Sie `(?i)` für case-insensitive)
bank-import-profile-labels-date-format = Datumsformat (chrono)
bank-import-profile-labels-decimal-separator = Dezimaltrennzeichen
bank-import-profile-labels-field-separator = Feldtrennzeichen
bank-import-profile-labels-encoding = Encoding (optional)
bank-import-profile-labels-header-row-count = Anzahl Header-Zeilen (0-5)
bank-import-profile-labels-column-mapping = Spaltenzuordnung (0-indiziert)
bank-import-profile-labels-use-debit-credit-split = Getrennte Soll-/Haben-Spalten
bank-import-profile-labels-create = Erstellen
bank-import-profile-labels-update = Aktualisieren
bank-import-profile-labels-edit = Bearbeiten
bank-import-profile-labels-delete = Löschen
bank-import-profile-labels-confirm-delete = Dieses Profil löschen?
bank-import-profile-labels-new-profile = Neues Profil
bank-import-profile-labels-no-profiles = Kein Bankprofil konfiguriert.
bank-import-profile-errors-bank-name-required = Bankname ist erforderlich.
bank-import-profile-errors-bank-name-duplicate = Ein Profil mit diesem Namen existiert bereits.
bank-import-profile-errors-column-mapping-xor-violation = Wählen Sie `amount` ODER `debit_credit_split`, nicht beide.
bank-import-profile-errors-date-format-invalid = Ungültiges chrono-Datumsformat.
bank-import-profile-errors-regex-invalid = Ungültige Regex für filename_pattern.
bank-import-profile-errors-separators-equal = Feld- und Dezimaltrennzeichen müssen unterschiedlich sein.

# Story 8-4 (FR44) — Bankabstimmung automatisch.
reconciliation-page-title = Abstimmung
reconciliation-page-subtitle = Automatische Vorschläge zum Abgleich Transaktion ↔ Rechnung.
reconciliation-labels-loading = Vorschläge werden geladen…
reconciliation-labels-empty = Keine Transaktionen zur Abstimmung vorhanden.
reconciliation-labels-no-account = Kein Bankkonto konfiguriert.
reconciliation-labels-account-select = Bankkonto
reconciliation-labels-no-candidate = Keine Übereinstimmung
reconciliation-labels-success-suffix = Vorgang/Vorgänge erfolgreich.
reconciliation-labels-failed = Teilfehler
reconciliation-cols-tx-date = Datum
reconciliation-cols-tx-amount = Betrag
reconciliation-cols-tx-counterparty = Gegenpartei
reconciliation-cols-candidate = Kandidat
reconciliation-cols-score = Score
reconciliation-actions-accept = Akzeptieren
reconciliation-actions-reject = Ablehnen
# H8 Pass 1 code review — 8 kanonische Schlüssel AC #61.
reconciliation-labels-validate-selected = Auswahl validieren
reconciliation-labels-reject-selected = Auswahl ablehnen
reconciliation-labels-score = Wert
reconciliation-errors-account-locked = Bankkonto wird gerade von einem anderen Benutzer abgeglichen. Bitte versuchen Sie es in wenigen Sekunden erneut.
reconciliation-errors-already-reconciled = Diese Transaktion wurde bereits abgeglichen.
reconciliation-errors-invoice-not-eligible = Diese Rechnung ist für den Abgleich nicht zulässig.
reconciliation-toast-accept-success = { $count } Transaktion(en) erfolgreich abgeglichen.
reconciliation-toast-reject-success = { $count } Transaktion(en) erfolgreich abgelehnt.

# Story 8-5a-base FR45 — Manueller Abgleich.
reconciliation-manual-button-label = Manuell zuweisen
reconciliation-manual-modal-title = Manueller Abgleich
reconciliation-manual-counterparty-label = Gegenkonto
reconciliation-manual-description-label = Bezeichnung
reconciliation-manual-bank-account-not-configured = Das Bankkonto ist nicht konfiguriert. Verknüpfen Sie das Buchhaltungskonto unter /bank-accounts.
reconciliation-manual-value-date-label = Valutadatum
reconciliation-manual-submit = Zuweisen
reconciliation-manual-error-no-proposal = Keine Transaktion ausgewählt
reconciliation-manual-error-counterparty-required = Gegenkonto erforderlich
reconciliation-manual-error-description-too-long = Bezeichnung zu lang (max. { $max } Zeichen)
reconciliation-manual-description-placeholder = Bankgebühren Mai

# Story 8-5a-bis FR48 — Aufteilung einer aggregierten Transaktion.
reconciliation-split-button-label = Aufteilen
reconciliation-split-modal-title = Transaktion aufteilen
reconciliation-split-balance-indicator = Saldo
reconciliation-split-error-imbalance = Die Aufteilung gleicht den Transaktionsbetrag nicht aus.

reconciliation-cols-actions = Aktionen

# Story 8-5a-zero — Verbindung `bank_account.journal_account_id`.
bank-accounts-labels-page-title = Bankkonten
bank-accounts-labels-page-subtitle = Jedes Bankkonto mit einem Konto des Kontorahmens verbinden (typisch Klasse 1: 1020 Kasse, 1030 Bank).
bank-accounts-labels-bank-name = Bank
bank-accounts-labels-iban = IBAN
bank-accounts-labels-journal-account-id = Verbundenes Buchhaltungskonto
bank-accounts-labels-not-configured = Nicht konfiguriert
bank-accounts-labels-empty = Keine Bankkonten konfiguriert.
bank-accounts-labels-loading = Wird geladen…
bank-accounts-actions-link-account = Mit Kontorahmen verbinden
bank-accounts-actions-unlink-account = Trennen
bank-accounts-actions-cancel = Abbrechen
bank-accounts-actions-submit = Verbinden
bank-accounts-errors-account-not-found = Buchhaltungskonto nicht gefunden.
bank-accounts-errors-invalid-account-type = Ungültiger Kontotyp (Aktiv- oder Passivkonto erforderlich).
# Story v014-1 — CRUD bank_accounts post-onboarding
bank-accounts-errors-has-transactions = Das Bankkonto enthält Transaktionen — Archivierung verweigert, um die Buchhaltungsprüfung zu wahren.
bank-accounts-errors-cannot-archive-primary = Das Hauptkonto kann nicht archiviert werden, solange ein anderes nicht-archiviertes Konto besteht. Setzen Sie zuerst ein anderes Konto als Hauptkonto und archivieren Sie dieses dann.
bank-accounts-errors-onboarding-not-complete = Das Onboarding muss abgeschlossen sein (Schritt 7), bevor Bankkonten verwaltet werden können.
# Story v014-1 — CRUD UI labels & actions (F3 Pass 1 code review parity DE/IT/EN)
bank-accounts-actions-create = Neues Bankkonto
bank-accounts-actions-edit = Bearbeiten
bank-accounts-actions-archive = Archivieren
bank-accounts-actions-confirm-archive = Archivieren
bank-accounts-actions-show-archived = Archivierte anzeigen
bank-accounts-actions-hide-archived = Archivierte ausblenden
bank-accounts-actions-submit-create = Erstellen
bank-accounts-actions-submit-update = Speichern
bank-accounts-labels-balance = Saldo
bank-accounts-labels-balance-unavailable = Saldo nicht verfügbar (mit Kontenrahmen verbinden)
bank-accounts-labels-qr-iban = QR-IBAN (optional)
bank-accounts-labels-is-primary = Hauptkonto
bank-accounts-labels-primary-badge = Hauptkonto
bank-accounts-labels-archived-badge = Archiviert
bank-accounts-confirm-archive = Archivierung dieses Bankkontos bestätigen? Diese Aktion ist in v0.1 unwiderruflich.
bank-accounts-tooltip-journal-account = Verbindet dieses Bankkonto mit einem Konto aus dem Kontenrahmen (typisch 1020 Kasse, 1030 Bank). Ermöglicht der automatischen Abstimmung, Buchungen auf das richtige Konto zu erstellen, und die Anzeige des Saldos auf der Startseite. Mehrere Konten: Wenn Sie mehrere separate Kontokorrente haben, verbinden Sie mit einem spezifischen Unterkonto (1030.001 BCV CHF), nicht mit dem übergeordneten Konto 1030.
bank-accounts-toast-create-success = Bankkonto erstellt.
bank-accounts-toast-update-success = Bankkonto geändert.
bank-accounts-toast-archive-success = Bankkonto archiviert.
# Story v014-1 — Homepage widget bank accounts (F14)
homepage-bank-total-liquidity = Liquide Mittel insgesamt
homepage-bank-total-partial = (nur verbundene Konten)
homepage-bank-balance-unavailable = Saldo nicht verfügbar — mit Kontenrahmen verbinden
homepage-bank-last-transaction = Letzte Transaktion
settings-bank-manage = In Administration → Bankkonten verwalten
settings-bank-manage-hint = Um ein Bankkonto hinzuzufügen, zu ändern oder zu archivieren, nutzen Sie die spezielle Seite Administration → Bankkonten.
bank-accounts-toast-link-success = Bankkonto erfolgreich mit dem Kontorahmen verbunden.
bank-accounts-toast-unlink-success = Bankkonto vom Kontorahmen getrennt.

# Story 8-5b — FR47 reconciliation rules. Traduction DE à compléter v0.2 (L51).
reconciliation-rules-page-title = Zuweisungsregeln
reconciliation-rules-loading = Lädt…
reconciliation-rules-labels-empty = Keine Regel konfiguriert.
reconciliation-rules-labels-label = Bezeichnung
reconciliation-rules-labels-match-type = Typ
reconciliation-rules-labels-match-value = Wert
reconciliation-rules-labels-counterparty-account = Gegenkonto
reconciliation-rules-labels-priority = Priorität
reconciliation-rules-labels-priority-hint = Kleinerer Wert = höhere Priorität (1-1000)
reconciliation-rules-labels-applied-count = Angewendet
reconciliation-rules-labels-status = Status
reconciliation-rules-labels-active = Aktiv
reconciliation-rules-labels-archived = Archiviert
reconciliation-rules-match-type-counterparty-contains = Gegenpartei enthält
reconciliation-rules-match-type-counterparty-exact = Gegenpartei exakt
reconciliation-rules-match-type-reference-contains = Referenz enthält
reconciliation-rules-match-type-iban-exact = IBAN exakt
reconciliation-rules-form-title-create = Neue Regel
reconciliation-rules-form-title-edit = Regel bearbeiten
reconciliation-rules-actions-new = Neue Regel
reconciliation-rules-actions-edit = Bearbeiten
reconciliation-rules-actions-create = Erstellen
reconciliation-rules-actions-save = Speichern
reconciliation-rules-actions-cancel = Abbrechen
reconciliation-rules-actions-archive = Archivieren
reconciliation-rules-actions-deactivate = Deaktivieren
reconciliation-rules-actions-reactivate = Reaktivieren
reconciliation-rules-confirm-delete = Diese Regel archivieren? Bereits angewendete Buchungen bleiben erhalten.
reconciliation-rules-error-label-required = Bezeichnung erforderlich
reconciliation-rules-error-match-value-required = Wert erforderlich
reconciliation-rules-error-counterparty-required = Gegenkonto erforderlich
reconciliation-rules-error-not-found = Regel nicht gefunden.
reconciliation-rules-error-duplicate = Eine aktive Regel existiert bereits für diese Typ/Wert-Kombination.
reconciliation-rules-applied-badge = Regel
reconciliation-rules-applied-score-na = Auto

# === Story 9-1 — Buchhaltungsberichte (34 Schlüssel) ===
# TODO official translation — basique livré dev-story (Pass 1 ECH-19)

reports-balance-sheet = Bilanz
reports-income-statement = Erfolgsrechnung
reports-trial-balance = Saldobilanz
reports-journals = Journale

reports-column-account-number = Konto-Nr.
reports-column-account-name = Bezeichnung
reports-column-debit = Soll
reports-column-credit = Haben
reports-column-balance = Saldo
reports-column-entry-date = Datum
reports-column-description = Buchungstext

reports-section-assets = Aktiven
reports-section-liabilities = Passiven
reports-section-equity = Eigenkapital
reports-section-revenues = Ertrag
reports-section-expenses = Aufwand

reports-total-assets = Total Aktiven
reports-total-liabilities = Total Passiven
reports-total-revenues = Total Ertrag
reports-total-expenses = Total Aufwand
reports-total-debit = Total Soll
reports-total-credit = Total Haben
reports-net-result = Periodenergebnis
reports-grand-total = Gesamttotal
# MwSt-Bericht (Story 11-2)
reports-vat = MwSt
reports-vat-column-rate = Satz
reports-vat-column-base-ht = Umsatz (netto)
reports-vat-column-vat-due = Geschuldete MwSt
reports-vat-total-base-ht = Total Umsatz netto
reports-vat-recoverable = Vorsteuer
reports-vat-balance = Saldo
reports-vat-reconciliation-warning = Die Abrechnung stimmt nicht mit den Buchungen überein (Differenz: { $delta }). Prüfen Sie manuell geänderte validierte Buchungen.

reports-filter-period = Periode
reports-filter-fiscal-year = Geschäftsjahr
reports-filter-journal = Journal
reports-button-generate = Erstellen

reports-error-no-entries-in-period = Keine Buchungen in der gewählten Periode. Ändern Sie die Daten oder wählen Sie ein anderes Geschäftsjahr.
reports-error-period-out-of-fiscal-year = Die gewählte Periode überschreitet die Grenzen des Geschäftsjahres. Wählen Sie eine Periode zwischen { $fyStart } und { $fyEnd }.
reports-error-no-fiscal-year-available = Kein Geschäftsjahr verfügbar. Erstellen Sie zuerst ein Geschäftsjahr.

reports-equity-result-section-title = Periodenergebnis (vor Abschlussbuchung)
reports-equity-result-profit = Gewinn der Periode
reports-equity-result-loss = Verlust der Periode
reports-retained-earnings = Gewinnvortrag
reports-retained-earnings-loss = Verlustvortrag
reports-trial-balance-period-note = Die Rohbilanz zeigt die Bewegung der Periode (pro Geschäftsjahr). Die Summe pro Konto ist nicht mit dem kumulierten Saldo desselben Kontos in der Bilanz vergleichbar (Saldovortrag seit Beginn).

# Alerts + badges UI (2 — code review Pass 1 i18n leaks)
reports-equation-warning = ⚠️ Bilanzgleichung ungültig (Quelldaten prüfen).
reports-archived-label = archiviert

# Berichte-Seite — chrome (3 — code review Pass 1 i18n leaks)
reports-page-title = Buchhaltungsberichte
reports-instruction-select-and-generate = Wählen Sie ein Geschäftsjahr aus und klicken Sie auf Generieren.
reports-loading = Bericht wird generiert…

# Story 9-2a — Export PDF & CSV (10 Schlüssel)
reports-export-pdf-button = PDF-Export
reports-export-csv-button = CSV-Export
# Story 21-7 — Altersstruktur der Forderungen
reports-aged-balance = Altersstruktur
reports-aged-instruction = Altersstruktur per heute.
reports-aged-instruction-generate = Klicken Sie auf Erstellen, um die Altersstruktur per heute anzuzeigen.
reports-aged-as-of = Stand { $date }
reports-aged-empty = Keine offenen Kundenforderungen.
reports-aged-col-contact = Kunde
reports-aged-col-not-due = Nicht fällig
reports-aged-col-1-30 = 1-30 T
reports-aged-col-31-60 = 31-60 T
reports-aged-col-61-90 = 61-90 T
reports-aged-col-over-90 = 90+ T
reports-aged-col-total = Total
reports-aged-total-row = Gesamttotal
reports-aged-link-due-dates = Fälligkeitsplan anzeigen
reports-export-loading = Datei wird generiert…
reports-export-error-generic = Bericht konnte nicht exportiert werden. Verbindung prüfen und erneut versuchen.
reports-filename-balance-sheet = bilanz
reports-filename-income-statement = erfolgsrechnung
reports-filename-trial-balance = kontensaldenliste
reports-filename-journals = journale
reports-filename-vat = mwst-abrechnung
reports-filename-project-expenses = ausgaben-pro-projekt
reports-filename-project-return = rendite-pro-projekt
reports-pdf-header-period = Zeitraum
reports-pdf-empty-message = Keine Buchungen im gewählten Zeitraum.

# Story 9-2b — Globaler ZIP-Export (Datensouveränität) — 12 Schlüssel
nav-export-global = Globaler Export
# Story 17-3b — vollständige Installationssicherung (.keshbackup, Admin)
nav-admin-backup = Vollständige Sicherung
admin-backup-page-title = Vollständige Sicherung der Installation
admin-backup-page-description = Lädt die gesamte Installation (alle Firmen, Benutzer und Systemdaten) in eine einzige .keshbackup-Datei zur Migration oder Sicherung herunter. Zu unterscheiden vom globalen Export einer einzelnen Firma.
admin-backup-action-export = Gesamte Installation exportieren
admin-backup-action-exporting = Export läuft…
admin-backup-toast-success = Installationssicherung heruntergeladen.
admin-backup-error-generic = Export der Installation fehlgeschlagen. Versuchen Sie es in Kürze erneut.
admin-backup-page-hint-secret = Die .keshbackup-Datei enthält sensible Daten (Anmeldedaten, Tokens). Bewahren Sie sie sicher auf.
# Story 17-3d — vollständiger Installationsimport / -wiederherstellung (.keshbackup, Admin)
nav-admin-restore = Wiederherstellen / Importieren
admin-restore-page-title = Wiederherstellung / Import der Installation
admin-restore-page-description = Laden Sie eine .keshbackup-Datei hoch, um die gesamte aktuelle Installation zu ersetzen (Migration oder Wiederherstellung). Destruktiver Vorgang: Vor dem Import wird serverseitig eine Sicherung des aktuellen Zustands erstellt.
admin-restore-file-label = Zu importierende .keshbackup-Datei
admin-restore-action-import = Importieren und Installation ersetzen
admin-restore-action-importing = Import läuft…
admin-restore-confirm-title = Gesamte Installation ersetzen?
admin-restore-confirm-body = Dieser Vorgang ersetzt ALLE Daten der aktuellen Installation. Vor dem Import wird serverseitig eine Sicherung des aktuellen Zustands erstellt. Sie werden abgemeldet und müssen sich mit den Anmeldedaten der importierten Instanz erneut anmelden.
admin-restore-confirm-cancel = Abbrechen
admin-restore-confirm-ok = Ersetzen bestätigen
admin-restore-toast-success = Import erfolgreich — Sie werden abgemeldet.
admin-restore-error-version = Dieses Backup erfordert eine neuere Kesh-Version ({ $src }) als die installierte ({ $bin }). Aktualisieren Sie Kesh vor dem erneuten Import.
admin-restore-error-schema = Backup-Schema ist mit dieser Kesh-Version nicht kompatibel (Tabelle { $table }).
admin-restore-error-invalid = Sicherungsdatei ungültig oder beschädigt. Stellen Sie sicher, dass es sich um eine von Kesh erzeugte .keshbackup-Datei handelt.
admin-restore-error-generic = Import fehlgeschlagen. Der vorherige Zustand der Installation wurde beibehalten.
export-global-title = Globaler Export Ihrer Daten
export-global-description = Exportieren Sie alle Ihre Buchhaltungsdaten (Konten, Buchungen, Kontakte, Rechnungen, Banktransaktionen) im CSV-Format in einer ZIP-Datei. Verwenden Sie diesen Export zur Archivierung, zur Migration in eine andere Software oder zur 10-jährigen Aufbewahrung (Schweizerisches OR Art. 958f).
export-global-button = Export starten
export-global-loading = Export wird erstellt…
export-global-success = Export heruntergeladen.
export-global-error-generic = Globaler Export konnte nicht erstellt werden. Überprüfen Sie Ihre Verbindung und versuchen Sie es erneut.
export-global-filename-hint = Die Datei wird unter dem Namen kesh-export-{ $companyShort }-{ $date }.zip heruntergeladen
export-global-content-includes = Der Export enthält: Kontenplan, Geschäftsjahre, Buchungen, Kontakte, Produkte, Rechnungen, Bankkonten, Bankimport-Historie, Transaktionen, aktive und historische Mehrwertsteuersätze, Rechnungseinstellungen, Abstimmungsregeln, Bankimport-Profile und ein metadata.json-Manifest mit SHA-256-Hash jeder Datei zur Integritätsprüfung.
export-global-content-excludes = Nicht enthalten: Benutzer (PII + Passwörter), Session-Tokens, internes Audit-Log, Onboarding-Status (Sicherheits- und technische Gründe).
export-global-souverainete-note = Ihre Daten gehören Ihnen. Kesh erstellt keine Kopien dieses Exports auf seinen Servern.
error-global-export-failed = Der globale Export konnte nicht erstellt werden. Wenn das Problem weiterhin besteht, wenden Sie sich an den Support.
error-admin-full-export-failed = Der vollständige Export der Installation konnte nicht erstellt werden. Versuchen Sie es in Kürze erneut; wenn das Problem weiterhin besteht, wenden Sie sich an den Support.
error-admin-full-import-failed = Der Import der Installation ist fehlgeschlagen. Der vorherige Zustand wurde beibehalten (vor dem Vorgang wurde automatisch ein Backup erstellt). Prüfen Sie die Serverprotokolle und versuchen Sie es erneut.
error-invalid-backup-structure = Die Sicherungsdatei ist ungültig oder beschädigt (unerwartete Struktur oder fehlgeschlagene Integritätsprüfung). Stellen Sie sicher, dass es sich um eine von Kesh erzeugte .keshbackup-Datei handelt.
error-import-schema-mismatch = Das Schema dieses Backups ist mit dieser Kesh-Version nicht kompatibel. Aktualisieren Sie Kesh oder verwenden Sie ein kompatibles Backup.
error-import-version-incompatible = Dieses Backup erfordert eine neuere Kesh-Version als die installierte. Aktualisieren Sie Kesh vor dem erneuten Import.

# Story v011-5 — Self-Service Onboarding (12 UI-Schlüssel + 2 Fehlerschlüssel)
error-setup-required = Erstkonfiguration erforderlich. Administratorkonto über /setup erstellen.
error-setup-already-complete = Das Administratorkonto wurde bereits erstellt.
setup-welcome = Willkommen bei Kesh
setup-intro = Um die Installation abzuschliessen, erstellen Sie das initiale Administratorkonto. Dieses Konto hat volle Rechte auf Ihrer Kesh-Instanz.
setup-username-label = Benutzername
setup-username-placeholder = admin
setup-username-required = Benutzername ist erforderlich.
setup-password-label = Passwort
setup-password-min = Mindestens 12 Zeichen.
setup-password-confirm-label = Passwort bestätigen
setup-password-mismatch = Die Passwörter stimmen nicht überein.
setup-email-label = E-Mail (empfohlen)
setup-email-hint = Ermöglicht das Zurücksetzen des Passworts per E-Mail, falls Sie es vergessen.
setup-email-invalid = Ungültiges E-Mail-Format.
setup-submit = Administratorkonto erstellen
setup-error-already-complete = Das Administratorkonto wurde bereits erstellt. Sie werden zur Anmeldeseite weitergeleitet.
setup-error-rate-limit = Zu viele Versuche. Bitte versuchen Sie es in einigen Minuten erneut.

# === Story 17-2b — API-Schlüssel (PAT) Frontend (36 Schlüssel) ===
# Einstellungen → Link
settings-api-keys-title = API-Schlüssel
settings-api-keys-manage = Verwalten
settings-api-keys-hint = Erstellen Sie API-Zugriffsschlüssel für Ihre Integrationen (externe KI, Skripte, Drittanbieter-Software).
# API-Schlüssel-Seite — Labels
api-keys-labels-page-title = API-Schlüssel
api-keys-labels-page-subtitle = Erstellen Sie API-Zugriffsschlüssel für Ihre Integrationen (externe KI, Skripte, Drittanbieter-Software). Übermitteln Sie den Schlüssel über den Header «Authorization: Bearer».
api-keys-labels-name = Name
api-keys-labels-name-placeholder = z. B. Buchhaltungsskript, KI-Agent…
api-keys-labels-scope = Berechtigung
api-keys-labels-scope-read = Nur Lesen
api-keys-labels-scope-read-write = Lesen und Schreiben
api-keys-labels-expires = Ablauf (optional)
api-keys-labels-expires-hint = Leer lassen für einen permanenten Schlüssel.
api-keys-labels-created-at = Erstellt am
api-keys-labels-last-used = Zuletzt verwendet
api-keys-labels-never-used = Nie verwendet
api-keys-labels-status = Status
api-keys-labels-status-active = Aktiv
api-keys-labels-status-expires = Aktiv (läuft ab am { $date })
api-keys-labels-status-revoked = Widerrufen am { $date }
api-keys-labels-status-expired = Abgelaufen am { $date }
api-keys-labels-empty = Keine API-Schlüssel. Erstellen Sie einen für Ihre Integrationen.
api-keys-labels-loading = Wird geladen…
api-keys-labels-secret-created = Schlüssel «{ $name }» erstellt.
api-keys-labels-secret-warning = Kopieren Sie diesen Schlüssel jetzt: Er wird nie wieder angezeigt.
# Aktionen
api-keys-actions-create = Neuer Schlüssel
api-keys-actions-submit-create = Schlüssel erstellen
api-keys-actions-cancel = Abbrechen
api-keys-actions-copy = Kopieren
api-keys-actions-close = Schliessen
api-keys-actions-revoke = Widerrufen
api-keys-actions-confirm-revoke = Widerrufen
# Bestätigung
api-keys-confirm-revoke = Diesen Schlüssel widerrufen? Jede Integration, die ihn verwendet, funktioniert sofort nicht mehr. Diese Aktion ist unwiderruflich.
# Fehler
api-keys-errors-name-required = Der Name des Schlüssels ist erforderlich.
api-keys-errors-name-too-long = Der Name des Schlüssels ist zu lang (maximal 255 Zeichen).
api-keys-errors-conflict = Der Schlüssel wurde zwischenzeitlich geändert — Liste neu geladen, bitte erneut versuchen.
# Toasts
api-keys-toast-create-success = API-Schlüssel erstellt.
api-keys-toast-copied = Schlüssel in die Zwischenablage kopiert.
api-keys-toast-copy-failed = Kopieren nicht möglich — bitte manuell markieren und kopieren.
api-keys-toast-revoke-success = Schlüssel widerrufen.

# Story 17-4b — Passwort-Wiederherstellung per E-Mail (Backend-Rendering, DC10)
error-smtp-send-failed = Der E-Mail-Versand ist fehlgeschlagen. Bitte versuchen Sie es in Kürze erneut.

# Story 20-3b1 — Rechnungsversand per E-Mail
error-smtp-not-configured = Der E-Mail-Versand ist auf dieser Instanz nicht konfiguriert (Variablen KESH_SMTP_*).
error-contact-email-missing = Der Kontakt der Rechnung hat keine E-Mail-Adresse. Bitte auf der Kontaktkarte erfassen.
error-invoice-email-empty-content = Betreff und Text der E-Mail dürfen nicht leer sein.
error-invoice-due-date-before-date = Das Fälligkeitsdatum darf nicht vor dem Rechnungsdatum liegen
error-contact-archived = Der Kontakt der Rechnung ist archiviert. Reaktivieren Sie ihn, bevor Sie die Rechnung per E-Mail senden.
error-email-sent-invoice-gone = Die E-Mail wurde dem Kontakt zugestellt, aber die Rechnung wurde zwischenzeitlich gelöscht — sie konnte nicht als « gesendet » markiert werden. Senden Sie die E-Mail nicht erneut.
error-company-email-invalid = Die E-Mail-Adresse des Unternehmens ist ungültig.
error-invalid-or-expired-token = Ungültiger oder abgelaufener Link zum Zurücksetzen des Passworts.
email-password-reset-subject = Zurücksetzen Ihres Kesh-Passworts
email-password-reset-body =
    Sie haben das Zurücksetzen Ihres Kesh-Passworts angefordert.
    Öffnen Sie den folgenden Link, um ein neues Passwort festzulegen (gültig für { $ttlMinutes } Minuten):
    { $resetUrl }
    Falls Sie diese Anfrage nicht gestellt haben, ignorieren Sie diese E-Mail.

# Story 17-4d — Passwort-Wiederherstellung (öffentliche Frontend-Seiten)
auth-recovery-forgot-title = Passwort vergessen
auth-recovery-forgot-intro = Geben Sie Ihren Benutzernamen oder Ihre E-Mail-Adresse ein. Falls ein Konto übereinstimmt, erhalten Sie einen Link zum Zurücksetzen.
auth-recovery-identifier-label = Benutzername oder E-Mail
auth-recovery-submit = Link zum Zurücksetzen senden
auth-recovery-success-generic = Falls ein Konto mit dieser Angabe übereinstimmt, wurde soeben eine E-Mail mit einem Link zum Zurücksetzen gesendet. Der Link ist 30 Minuten gültig.
auth-recovery-error-rate-limit = Zu viele Versuche. Versuchen Sie es in einigen Minuten erneut.
auth-recovery-error-network = Server nicht erreichbar. Überprüfen Sie Ihre Verbindung.
auth-recovery-error-unavailable = Das Zurücksetzen per E-Mail ist nicht verfügbar. Wenden Sie sich an Ihren Administrator.
auth-recovery-error-server = Serverfehler. Versuchen Sie es später erneut.
auth-recovery-back-to-login = Zurück zur Anmeldung
auth-recovery-reset-title = Neues Passwort
auth-recovery-reset-intro = Wählen Sie Ihr neues Passwort.
auth-recovery-new-password-label = Neues Passwort
auth-recovery-password-confirm-label = Passwort bestätigen
auth-recovery-password-min = Mindestens 12 Zeichen.
auth-recovery-password-mismatch = Die Passwörter stimmen nicht überein.
auth-recovery-reset-submit = Passwort zurücksetzen
auth-recovery-reset-success = Ihr Passwort wurde zurückgesetzt. Sie können sich jetzt anmelden.
auth-recovery-invalid-link = Dieser Link zum Zurücksetzen ist ungültig oder abgelaufen. Stellen Sie eine neue Anfrage, um einen neuen Link zu erhalten.
auth-recovery-request-new-link = Neue Anfrage stellen
auth-recovery-login-cta = Anmelden

# Story 12.2 — factures fournisseurs (#191)
supplier-invoices-title = Lieferantenrechnungen

# Story 12.3 — paiements pain.001 (#191)
payment-batches-title = Lieferantenzahlungen

# Story 20-3b2 — Rechnungsversand per E-Mail (UI)
common-save = Speichern
common-edit = Bearbeiten
error-unexpected = Unerwarteter Fehler.
invoice-send-email-button = Per E-Mail senden
invoice-resend-email-button = Erneut per E-Mail senden
invoice-send-email-smtp-tooltip = Der E-Mail-Versand ist nicht konfiguriert (KESH_SMTP_*-Variablen) — siehe Administratorhandbuch.
invoice-send-email-title = Rechnung per E-Mail senden
invoice-send-email-to-label = Empfänger
invoice-send-email-to-missing = Der Kontakt hat keine E-Mail-Adresse — erfassen Sie sie auf der Kontaktseite.
invoice-send-email-subject-label = Betreff
invoice-send-email-body-label = Nachricht
invoice-send-email-confirm = E-Mail senden
invoice-send-email-success = Rechnung per E-Mail gesendet
invoice-send-email-error-empty = Betreff und Text der E-Mail dürfen nicht leer sein.
invoice-detail-emailed-at-label = Gesendet am
contact-form-language = Korrespondenzsprache
contact-form-language-inherited = Geerbt (Sprache der Instanz)
contact-form-salutation = Anrede
contact-salutation-neutre = Neutral
contact-salutation-monsieur = Herr
contact-salutation-madame = Frau
settings-field-company-email = E-Mail (Antwortadresse)
settings-company-email-help = Antwortadresse (Reply-To) der per E-Mail gesendeten Rechnungen. Leer = keine Antwortadresse.
settings-company-email-invalid = Ungültige E-Mail-Adresse.
settings-company-email-saved = Firmen-E-Mail gespeichert
settings-company-email-conflict = Versionskonflikt — die Daten wurden neu geladen, bitte erneut versuchen.
settings-company-email-conflict-reload-failed = Versionskonflikt und Neuladen fehlgeschlagen — laden Sie die Seite neu.

# --- Mahnwesen (Story 21-4, #231) ---
dunning-title = Mahnungen
dunning-subtitle = Konfigurieren Sie die Mahnstufen, Fristen und Mahngebühren.
dunning-load-error = Mahneinstellungen konnten nicht geladen werden.
dunning-grace-heading = Karenzfrist
dunning-grace-help = Tage nach Fälligkeit, bevor die 1. Mahnung fällig wird.
dunning-grace-label = Karenz (Tage)
dunning-grace-save = Speichern
dunning-grace-saved = Karenzfrist gespeichert.
dunning-levels-heading = Mahnstufen
dunning-level-new = Stufe hinzufügen
dunning-empty = Keine Stufe konfiguriert — Mahnungen sind deaktiviert.
dunning-col-level = Stufe
dunning-col-delay = Frist (Tage)
dunning-col-fee = Gebühr (CHF)
dunning-col-actions = Aktionen
dunning-edit = Bearbeiten
dunning-delete = Löschen
dunning-example-heading = Voraussichtlicher Zeitplan
dunning-example-line = { $level }. Mahnung { $days } Tage nach Fälligkeit vorgeschlagen
dunning-cgv-hint = Mahngebühren sind nur mit einer vertraglichen Grundlage (AGB) geschuldet. Sie sind nicht im QR der beigefügten Rechnung enthalten.
dunning-delay-label = Frist (Tage)
dunning-delay-help = Tage seit dem vorherigen Schritt (Fälligkeit + Karenz für die 1.).
dunning-fee-label = Gebühr (CHF)
dunning-form-submit = Speichern
dunning-form-cancel = Abbrechen
dunning-form-error = Speichern fehlgeschlagen.
dunning-form-error-delay = Die Frist muss eine positive ganze Zahl sein.
dunning-form-error-fee = Die Gebühr muss ein gültiger Betrag sein.
dunning-delete-confirm-body = Diese Mahnstufe löschen? Die folgenden Stufen werden neu nummeriert.
dunning-delete-confirm-action = Löschen
dunning-created = Mahnstufe hinzugefügt.
dunning-updated = Mahnstufe aktualisiert.
dunning-deleted = Mahnstufe gelöscht.
dunning-conflict = Die Stufe wurde zwischenzeitlich geändert, neu geladen.
settings-dunning-link = Konfigurieren Sie die Mahnstufen (Fristen und Gebühren), die Karenzfrist und passen Sie die Mahn-E-Mail-Texte pro Stufe an.
email-templates-type-invoice_send = Rechnungsversand
email-templates-type-invoice_reminder = Zahlungserinnerung
email-templates-level-generic = Allgemein
email-templates-level-n = Mahnung { $n }
email-templates-type-label = Typ
email-templates-level-label = Stufe

# Story 21-6b — Mahnungen (Versand der Debitorenmahnungen)
reminders-page-title = Mahnungen
reminders-forbidden = Zugriff nur für Buchhalter und Administratoren.
reminders-empty = Keine Rechnung zu mahnen.
reminders-level-name = Mahnung { $level }
reminders-level-next = Nächste: Mahnung { $level }
reminders-last-sent = letzte am { $date }
reminders-select-invoice = { $inv } auswählen
reminders-selected-count = { $n } ausgewählt
reminders-batch-cap = Maximal { $cap } Rechnungen pro Stapel.
reminders-batch-send = Ausgewählte Mahnungen senden
reminders-sending = Senden…
reminders-saving = Speichern…
reminders-badge-no-email = keine E-Mail
reminders-badge-terminal = Letzte Stufe erreicht
reminders-batch-accepted = { $n } Mahnung(en) gesendet.
reminders-batch-failed = { $n } Fehler:
reminders-send-title = Mahnung senden
reminders-send-open = Mahnung senden
reminders-send-level-label = Mahnstufe
reminders-send-to-label = Empfänger
reminders-send-no-recipient = Der Kontakt hat keine E-Mail-Adresse.
reminders-send-subject-label = Betreff
reminders-send-body-label = Nachricht
reminders-send-empty = Betreff und Text dürfen nicht leer sein.
reminders-send-confirm = Mahnung senden
reminders-send-success = Mahnung gesendet
reminders-manual-title = Manuelle Mahnung erfassen
reminders-manual-open = Manuelle Mahnung
reminders-manual-body = Erfassen Sie eine bereits ausserhalb von Kesh versandte Mahnung (Brief, Einschreiben). Es wird keine E-Mail gesendet.
reminders-manual-level-label = Mahnstufe
reminders-manual-date-label = Versanddatum
reminders-manual-date-required = Versanddatum erforderlich
reminders-manual-date-future = Das Versanddatum darf nicht in der Zukunft liegen
reminders-manual-note-label = Notiz (optional)
reminders-manual-confirm = Speichern
reminders-manual-success = Manuelle Mahnung erfasst
reminders-error-invoice-not-found = Rechnung nicht gefunden
reminders-error-invoice-not-validated = Rechnung nicht validiert
reminders-error-invoice-already-paid = Rechnung bereits bezahlt
reminders-error-dunning-paused = Mahnungen ausgesetzt
reminders-error-no-next-level = Letzte Stufe erreicht
reminders-error-contact-archived = Kontakt archiviert
reminders-error-contact-email-missing = Kontakt ohne E-Mail-Adresse
reminders-error-content-empty = Mahnvorlage leer
reminders-error-content-too-long = Mahninhalt zu lang
reminders-error-not-pdf-ready = Rechnung nicht als PDF druckbar
reminders-error-rate-limited = Sendelimit erreicht
reminders-error-database-error = Technischer Fehler
reminders-error-smtp-failed = E-Mail-Versand fehlgeschlagen
reminders-error-sent-but-gone = E-Mail gesendet, aber die Rechnung ist zwischenzeitlich verschwunden (nicht erfasst)
reminders-error-sent-not-recorded = E-Mail gesendet, aber nicht erfasst (technischer Fehler)
reminders-error-unknown = Fehler ({ $code })
# Story 21-6c — Verlauf & Aussetzung auf der Rechnungsdetailseite
reminders-history-title = Mahnungsverlauf
reminders-history-empty = Keine Mahnung versendet.
reminders-history-col-date = Datum
reminders-history-col-level = Stufe
reminders-history-col-channel = Kanal
reminders-history-col-recipient = Empfänger
reminders-history-col-fee = Gebühr
reminders-history-channel-email = E-Mail
reminders-history-channel-manual = Manuell
reminders-history-cancelled-at = Storniert am { $date }
reminders-pause-button = Mahnungen aussetzen
reminders-resume-button = Mahnungen fortsetzen
reminders-pause-title = Mahnungen aussetzen
reminders-pause-body = Die automatischen Mahnungen dieser Rechnung werden bis zur Fortsetzung ausgesetzt. Sie können den Grund vermerken (Streitfall, Vereinbarung).
reminders-pause-note-label = Grund (optional)
reminders-pause-confirm = Aussetzen
reminders-pause-submitting = Wird ausgesetzt…
reminders-pause-success = Mahnungen ausgesetzt
reminders-resume-success = Mahnungen fortgesetzt
reminders-error-not-paused = Diese Rechnung ist nicht mehr ausgesetzt.
reminders-link-due-dates = Fälligkeitsplan anzeigen
reminders-link-aged = Altersstruktur anzeigen

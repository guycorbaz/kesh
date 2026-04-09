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

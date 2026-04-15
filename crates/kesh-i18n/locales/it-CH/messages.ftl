# Kesh — Messaggi italiano (Svizzera)

# Errori di autenticazione
error-invalid-credentials = Credenziali non valide
error-unauthenticated = Non autenticato
error-invalid-refresh-token = Sessione scaduta
error-rate-limited = Troppi tentativi

# Errori di autorizzazione
error-forbidden = Accesso negato
error-cannot-disable-self = Impossibile disattivare il proprio account
error-cannot-disable-last-admin = Impossibile disattivare l'ultimo amministratore

# Errori di risorsa
error-not-found = Risorsa non trovata
error-conflict = Risorsa già esistente
error-optimistic-lock = Conflitto di versione — la risorsa è stata modificata
error-foreign-key = Riferimento non valido
error-check-constraint = Valore non valido
error-illegal-state = Transizione di stato non consentita

# Errori di validazione
error-validation = Errore di validazione
error-username-empty = Il nome utente non può essere vuoto
error-username-too-long = Il nome utente non deve superare { $max } caratteri

# Errori di sistema
error-internal = Errore interno
error-service-unavailable = Servizio temporaneamente non disponibile

# Errori onboarding (Story 2.2)
error-onboarding-step-already-completed = Questo passaggio di configurazione è già stato completato

# Onboarding — procedura guidata
onboarding-choose-mode = Scegli la tua modalità di utilizzo
onboarding-mode-guided = Guidato
onboarding-mode-guided-desc = Spaziature generose, aiuto contestuale, conferme prima delle azioni
onboarding-mode-expert = Esperto
onboarding-mode-expert-desc = Interfaccia compatta, scorciatoie da tastiera, azioni dirette
onboarding-choose-path = Come vuoi iniziare?
onboarding-path-demo = Esplora con dati dimostrativi
onboarding-path-demo-desc = Scopri Kesh con dati fittizi realistici
onboarding-path-production = Configura per la produzione
onboarding-path-production-desc = Configura la tua organizzazione per iniziare a lavorare

# Banner demo
demo-banner-text = Istanza dimostrativa — dati fittizi
demo-banner-reset = Reimposta per la produzione
demo-reset-confirm-title = Reimposta l'istanza
demo-reset-confirm-body = Tutti i dati dimostrativi verranno eliminati. Vuoi continuare?
demo-reset-confirm-ok = Conferma
demo-reset-confirm-cancel = Annulla

# Onboarding — Percorso B (Story 2.3)
onboarding-choose-org-type = Tipo di organizzazione
onboarding-org-independant = Indipendente
onboarding-org-independant-desc = Lavoratore indipendente, freelance
onboarding-org-association = Associazione
onboarding-org-association-desc = Associazione senza scopo di lucro
onboarding-org-pme = PMI
onboarding-org-pme-desc = Piccola e media impresa (SA, Sagl)
onboarding-choose-accounting-lang = Lingua contabile
onboarding-accounting-lang-desc = Lingua delle denominazioni del piano dei conti (indipendente dalla lingua dell'interfaccia)
onboarding-coordinates-title = Dati della vostra organizzazione
onboarding-field-name = Nome / Ragione sociale
onboarding-field-address = Indirizzo
onboarding-field-ide = Numero IDI
onboarding-field-ide-hint = opzionale, formato CHE-xxx.xxx.xxx
onboarding-bank-title = Conto bancario principale
onboarding-field-bank-name = Nome della banca
onboarding-field-iban = IBAN
onboarding-field-qr-iban = QR-IBAN
onboarding-skip-bank = Configurare più tardi
onboarding-next = Continua
incomplete-banner-text = Configurazione incompleta — Completare la configurazione
incomplete-banner-cta = Completare la configurazione

# Pagina iniziale (Story 2.4)
homepage-title = Panoramica
homepage-entries-title = Ultime registrazioni
homepage-entries-empty = Nessuna registrazione.
homepage-entries-empty-guided = Nessuna registrazione per il momento. Iniziate inserendo la vostra prima registrazione contabile.
homepage-entries-action = Inserire una registrazione
homepage-invoices-title = Fatture aperte
homepage-invoices-empty = Nessuna fattura aperta.
homepage-invoices-empty-guided = Nessuna fattura aperta. Create la vostra prima fattura per fatturare ai vostri clienti.
homepage-invoices-action = Creare una fattura
homepage-bank-title = Conti bancari
homepage-bank-empty = Nessun conto bancario.
homepage-bank-empty-guided = Nessun conto bancario configurato. Aggiungete il vostro conto per importare gli estratti conto.
homepage-bank-no-transactions = Nessuna transazione importata
homepage-bank-action = Configurare

# Impostazioni (Story 2.4)
settings-title = Impostazioni
settings-org-title = Organizzazione
settings-accounting-title = Contabilità
settings-bank-title = Conti bancari
settings-users-title = Utenti
settings-field-name = Nome
settings-field-address = Indirizzo
settings-field-ide = IDI
settings-field-org-type = Tipo di organizzazione
settings-field-instance-language = Lingua dell'interfaccia
settings-field-accounting-language = Lingua contabile
search-coming-soon = Ricerca presto disponibile

# Misc i18n (Story 2.4 review)
loading = Caricamento...
settings-edit = Modificare
settings-edit-coming-soon = Modifica presto disponibile
settings-manage = Gestire
settings-no-bank = Nessun conto bancario configurato.
settings-no-company = Nessuna organizzazione configurata. Completare l'onboarding.

# Piano dei conti (Story 3.1)
accounts-title = Piano dei conti
accounts-add = Nuovo conto
accounts-edit = Modifica conto
accounts-archive = Archiviare
accounts-archive-confirm = Il conto non sarà più disponibile nelle selezioni future, ma rimarrà visibile nelle registrazioni esistenti.
account-field-number = Numero
account-field-name = Nome
account-field-type = Tipo
account-field-parent = Conto superiore
account-type-asset = Attivo
account-type-liability = Passivo
account-type-revenue = Ricavo
account-type-expense = Costo
account-archived-label = Archiviato

# Modalità Guidato/Esperto (Story 2.5)
mode-guided-label = Guidato
mode-expert-label = Esperto
shortcut-new-entry = Ctrl+N : Nuova registrazione

# Scritture contabili (Story 3.2)
error-entry-unbalanced = Scrittura non bilanciata — il totale degli addebiti ({ $debit }) non corrisponde al totale degli accrediti ({ $credit })
error-no-fiscal-year = Nessun esercizio esiste per la data { $date }. Crea un esercizio contabile prima di inserire scritture.
error-fiscal-year-closed = L'esercizio per la data { $date } è chiuso — nessuna scrittura può essere aggiunta o modificata (CO art. 957-964).
journal-entries-title = Scritture contabili
journal-entries-new = Nuova scrittura
journal-entries-empty-list = Nessuna scrittura inserita
journal-entries-col-number = N°
journal-entries-col-date = Data
journal-entries-col-journal = Giornale
journal-entries-col-description = Descrizione
journal-entries-col-total = Totale
journal-entry-form-title = Inserimento scrittura
journal-entry-form-date = Data
journal-entry-form-journal = Giornale
journal-entry-form-description = Descrizione
journal-entry-form-add-line = + Aggiungi riga
journal-entry-form-remove-line = Rimuovi riga
journal-entry-form-col-account = Conto
journal-entry-form-col-debit = Dare
journal-entry-form-col-credit = Avere
journal-entry-form-total-debit = Totale Dare
journal-entry-form-total-credit = Totale Avere
journal-entry-form-diff = Differenza
journal-entry-form-balanced = Bilanciato
journal-entry-form-unbalanced = Non bilanciato
journal-entry-form-submit = Salva
journal-entry-form-cancel = Annulla
journal-entry-form-incomplete-line = Riga incompleta
journal-entry-form-max-decimals = Massimo 4 decimali
journal-entry-form-amount-too-large = Importo troppo elevato
account-autocomplete-unavailable = Completamento automatico non disponibile — inserire l'ID del conto
journal-achats = Acquisti
journal-ventes = Vendite
journal-banque = Banca
journal-caisse = Cassa
journal-od = Operazioni diverse
journal-entry-saved = Scrittura salvata
error-fiscal-year-closed-generic = L'esercizio contabile è chiuso — nessuna scrittura può essere aggiunta o modificata (CO art. 957-964).
error-inactive-accounts = Uno o più conti sono archiviati o non validi.

# Modifica & eliminazione scritture (Story 3.3)
journal-entry-edit = Modifica
journal-entry-delete = Elimina
journal-entry-delete-confirm-title = Eliminare la scrittura N°{ $number }?
journal-entry-delete-confirm-message = Questa azione è irreversibile. L'azione verrà registrata nel registro di audit.
journal-entry-delete-confirm-cancel = Annulla
journal-entry-delete-confirm-delete = Elimina
journal-entry-deleted = Scrittura eliminata
journal-entry-conflict-title = Conflitto di versione
journal-entry-conflict-message = Questa scrittura è stata modificata da un altro utente. Ricaricare?
journal-entry-conflict-reload = Ricarica
journal-entry-conflict-reloaded = Lista ricaricata — cliccare nuovamente su Modifica
error-date-outside-fiscal-year = La data { $date } non rientra nell'esercizio corrente di questa scrittura
error-date-outside-fiscal-year-generic = La data non rientra nell'esercizio corrente di questa scrittura

# Ricerca, paginazione, ordinamento (Story 3.4)
journal-entries-filter-description = Descrizione
journal-entries-filter-amount-min = Importo min
journal-entries-filter-amount-max = Importo max
journal-entries-filter-date-from = Data inizio
journal-entries-filter-date-to = Data fine
journal-entries-filter-journal = Giornale
journal-entries-filter-journal-all = Tutti
journal-entries-filter-reset = Reimposta
journal-entries-pagination-on = su
journal-entries-pagination-prev = Precedente
journal-entries-pagination-next = Successivo
journal-entries-pagination-page-size = Per pagina
journal-entries-sort-asc-indicator = ordinamento crescente
journal-entries-sort-desc-indicator = ordinamento decrescente
journal-entries-loading = Caricamento…

# Tooltip bilingue termini contabili (Story 3.5)
tooltip-debit-natural = Il denaro entra in questo conto
tooltip-debit-technical = Dare — colonna sinistra
tooltip-credit-natural = Il denaro esce da questo conto
tooltip-credit-technical = Avere — colonna destra
tooltip-journal-natural = Registro in cui sono raggruppate le scritture simili
tooltip-journal-technical = Giornale contabile (Acquisti, Vendite, Banca, Cassa, Operazioni diverse)
tooltip-balanced-natural = Il totale delle entrate è uguale al totale delle uscite
tooltip-balanced-technical = Equilibrio della partita doppia (Dare = Avere)

# Story 4.1 — Rubrica (contatti CRUD)
nav-contacts = Rubrica
contacts-page-title = Rubrica
contact-form-create-title = Nuovo contatto
contact-form-edit-title = Modifica contatto
contact-form-name = Nome / Ragione sociale
contact-form-type = Tipo
contact-form-is-client = Cliente
contact-form-is-supplier = Fornitore
contact-form-email = E-mail
contact-form-phone = Telefono
contact-form-address = Indirizzo
contact-form-ide = Numero IDI (CHE)
contact-form-ide-help = Formato: CHE-123.456.789
contact-type-personne = Persona
contact-type-entreprise = Impresa
contact-form-submit-create = Crea
contact-form-submit-edit = Salva
contact-form-cancel = Annulla
contact-list-new = Nuovo contatto
contact-list-edit = Modifica
contact-list-archive = Archivia
contact-archive-confirm = Archivia
contact-archive-cancel = Annulla
contact-col-name = Nome
contact-col-type = Tipo
contact-col-flags = Ruoli
contact-col-ide = IDI
contact-col-email = E-mail
contact-col-actions = Azioni
contact-filter-search-placeholder = Cerca per nome o e-mail…
contact-filter-type-all = Tutti i tipi
contact-filter-archived = Includi archiviati
contact-empty-list = Nessun contatto. Crea il tuo primo contatto con il pulsante « Nuovo contatto ».
contact-created-success = Contatto creato
contact-updated-success = Contatto aggiornato
contact-archived-success = Contatto archiviato
contact-archive-confirm-title = Archiviare il contatto?
contact-archive-confirm-body = Il contatto non sarà più visibile nell'elenco predefinito. Potrai comunque consultarlo attivando « Includi archiviati ».
contact-error-name-required = Il nome è obbligatorio
contact-error-name-too-long = Il nome deve contenere al massimo 255 caratteri
contact-error-email-invalid = Formato e-mail non valido
contact-error-ide-invalid = Numero IDI svizzero non valido (formato o checksum)
contact-error-ide-duplicate = Esiste già un contatto con questo numero IDI
contact-error-not-found = Contatto non trovato
contact-error-archived-no-modify = Contatto archiviato — modifica o ulteriore archiviazione vietata
contact-conflict-title = Conflitto di versione
contact-conflict-body = Questo contatto è stato modificato altrove. Vuoi ricaricare la versione attuale?
error-ide-already-exists = Esiste già un contatto con questo numero IDI

# Story 4.2 — Condizioni di pagamento e catalogo prodotti
contact-form-payment-terms = Condizioni di pagamento
contact-form-payment-terms-placeholder = es. 30 giorni netti
products-page-title = Catalogo prodotti/servizi
product-form-create-title = Nuovo prodotto
product-form-edit-title = Modifica prodotto
product-form-name = Nome
product-form-description = Descrizione
product-form-price = Prezzo unitario
product-form-vat-rate = Aliquota IVA
product-form-vat-help = Aliquote svizzere in vigore dal 01.01.2024
product-vat-exempt = 0,00 % — Esente
product-vat-reduced = 2,60 % — Aliquota ridotta
product-vat-special = 3,80 % — Alloggio
product-vat-normal = 8,10 % — Aliquota normale
product-list-new = Nuovo prodotto
product-list-edit = Modifica
product-list-archive = Archivia
product-col-name = Nome
product-col-description = Descrizione
product-col-price = Prezzo
product-col-vat = IVA
product-col-actions = Azioni
product-filter-search = Cerca per nome o descrizione…
product-filter-archived = Includi archiviati
product-empty-list = Nessun prodotto. Creane il primo con « Nuovo prodotto ».
product-created-success = Prodotto creato
product-updated-success = Prodotto modificato
product-archived-success = Prodotto archiviato
product-error-name-required = Il nome è obbligatorio
product-error-name-too-long = Il nome può avere al massimo 255 caratteri
product-error-price-required = Il prezzo è obbligatorio
product-error-price-negative = Il prezzo deve essere positivo o zero
product-error-price-invalid = Formato del prezzo non valido
product-error-vat-invalid = Aliquota IVA non consentita
product-error-name-duplicate = Esiste già un prodotto con questo nome
product-archive-confirm-title = Archiviare il prodotto?
product-archive-confirm-body = Il prodotto non sarà più visibile nell'elenco predefinito. Potrai consultarlo attivando « Includi archiviati ».
product-conflict-title = Conflitto di versione
product-conflict-body = Questo prodotto è stato modificato altrove. Vuoi ricaricare la versione attuale?
product-filter-reset = Reimposta
product-pagination-prev = Precedente
product-pagination-next = Successivo
product-pagination-of = su
product-conflict-reload = Ricarica
product-form-cancel = Annulla
product-form-submit-create = Crea
product-form-submit-edit = Salva
product-archive-cancel = Annulla
product-archive-confirm = Archivia

# --- Story 5.1: Bozze fatture ---
nav-invoices = Fatture
invoices-page-title = Fatture
invoice-new-title = Nuova fattura
invoice-edit-title = Modifica fattura
invoice-view-title = Fattura
invoice-form-contact = Contatto
invoice-form-date = Data
invoice-form-due-date = Scadenza
invoice-form-payment-terms = Condizioni di pagamento
invoice-form-status = Stato
invoice-form-number = N. fattura
invoice-line-description = Descrizione
invoice-line-quantity = Quantità
invoice-line-unit-price = Prezzo unitario
invoice-line-vat-rate = IVA %
invoice-line-total = Totale
invoice-line-actions = Azioni
invoice-add-free-line = Riga libera
invoice-add-from-catalog = Dal catalogo
invoice-col-date = Data
invoice-col-contact = Contatto
invoice-col-number = N.
invoice-col-status = Stato
invoice-col-total = Totale
invoice-col-actions = Azioni
invoice-status-draft = Bozza
invoice-status-validated = Convalidata
invoice-status-cancelled = Annullata
invoice-filter-search = Cerca…
invoice-filter-status-all = Tutti gli stati
invoice-filter-contact-all = Tutti i contatti
invoice-filter-date-from = Da
invoice-filter-date-to = A
invoice-new-button = Nuova fattura
invoice-edit-button = Modifica
invoice-delete-button = Elimina
invoice-subtotal = Subtotale
invoice-total = Totale
invoice-empty-list = Nessuna fattura. Crea la prima con «Nuova fattura».
invoice-created-success = Fattura creata
invoice-updated-success = Fattura modificata
invoice-deleted-success = Fattura eliminata
invoice-delete-confirm-title = Eliminare la fattura?
invoice-delete-confirm-body = Questa fattura in bozza sarà eliminata definitivamente.
invoice-conflict-title = Conflitto di versione
invoice-conflict-body = Questa fattura è stata modificata altrove. Ricaricare la versione attuale?
invoice-error-no-lines = Una fattura deve contenere almeno una riga
invoice-error-contact-required = Selezionare un contatto
invoice-error-contact-invalid = Contatto non trovato
invoice-error-quantity-positive = La quantità deve essere strettamente positiva
invoice-error-description-required = La descrizione è obbligatoria
invoice-error-vat-invalid = Aliquota IVA non consentita. Valori: 0.00%, 2.60%, 3.80%, 8.10%
invoice-error-illegal-state = Questa fattura non può più essere modificata
invoice-product-picker-title = Seleziona prodotto
invoice-product-picker-search = Cerca prodotto…
invoice-product-picker-empty = Nessun prodotto
invoice-contact-picker-placeholder = Cerca contatto…
invoice-contact-picker-empty = Nessun contatto

# Story 5.2 — Validation & numérotation (TODO: traduction — fallback fr-CH via kesh-i18n)

# --- Story 5.3 — PDF QR-fattura ---

invoice-pdf-title = Fattura
invoice-pdf-date = Data
invoice-pdf-due-date = Scadenza
invoice-pdf-number = N° fattura
invoice-pdf-ide = IDI
invoice-pdf-recipient = Destinatario
invoice-pdf-description = Descrizione
invoice-pdf-quantity = Qtà
invoice-pdf-unit-price = Prezzo unitario
invoice-pdf-vat = IVA
invoice-pdf-line-total = Totale
invoice-pdf-subtotal = Subtotale
invoice-pdf-total = Totale
invoice-pdf-total-ttc = Totale IVA incl.
invoice-pdf-payment-terms = Condizioni di pagamento
invoice-pdf-qr-section-payment = Sezione pagamento
invoice-pdf-qr-section-receipt = Ricevuta
invoice-pdf-qr-account = Conto / Pagabile a
invoice-pdf-qr-reference = Riferimento
invoice-pdf-qr-additional-info = Informazioni supplementari
invoice-pdf-qr-payable-by = Pagabile da
invoice-pdf-qr-currency = Valuta
invoice-pdf-qr-amount = Importo
invoice-pdf-qr-acceptance-point = Punto di accettazione
invoice-pdf-qr-separate-before-paying = Da staccare prima del versamento

invoice-pdf-error-invoice-not-validated = La fattura deve essere convalidata prima di generare il PDF.
invoice-pdf-error-invoice-not-pdf-ready = La fattura non è pronta per la generazione PDF.
invoice-pdf-error-pdf-generation-failed = Generazione PDF fallita. Riprovare più tardi.
invoice-pdf-error-popup-blocked = Pop-up bloccato dal browser — consentire i pop-up per scaricare il PDF.
invoice-pdf-error-missing-contact-address = Indirizzo del cliente mancante — compilare la scheda contatto.
invoice-pdf-error-missing-primary-bank-account = Nessun conto bancario principale configurato — aggiungerlo nelle impostazioni.

invoices-download-pdf = Scarica PDF
invoices-download-pdf-aria-label = Scarica la fattura { $number } in formato PDF

error-invoice-not-validated = La fattura deve essere convalidata prima di generare il PDF.
error-invoice-too-many-lines-for-pdf = La fattura contiene { $count } righe — il PDF A4 è limitato a { $max } righe in v0.1.
error-pdf-generation-failed = Generazione PDF fallita.

# Story 5.4 — Scadenziario fatture
due-dates-title = Scadenziario
due-dates-filter-all = Tutte
due-dates-filter-unpaid = Non pagate
due-dates-filter-overdue = In ritardo
due-dates-filter-paid = Pagate
due-dates-summary-unpaid = fatture non pagate
due-dates-summary-overdue = in ritardo
due-dates-search-label = Ricerca
due-dates-contact-label = Contatto
due-dates-contact-placeholder = Tutti i contatti
due-dates-due-before-label = Scadenza entro
due-dates-column-date = Data
due-dates-column-due-date = Scadenza
due-dates-column-contact = Cliente
due-dates-column-total = Totale
due-dates-column-payment-status = Stato
due-dates-column-paid-at = Pagata il
due-dates-export-button = Esporta CSV
due-dates-no-results = Nessuna fattura da mostrare.
due-dates-result-suffix = risultato/i

payment-status-paid = Pagata
payment-status-unpaid = Non pagata
payment-status-overdue = In ritardo

invoice-mark-paid-button = Segna come pagata
invoice-mark-paid-dialog-title = Segna la fattura come pagata
invoice-mark-paid-dialog-body = Indica la data in cui hai ricevuto il pagamento.
invoice-mark-paid-date-label = Data di pagamento
invoice-mark-paid-confirm = Conferma pagamento
invoice-mark-paid-success = Fattura segnata come pagata
invoice-unmark-paid-button = Annulla pagamento
invoice-unmark-paid-dialog-title = Annulla pagamento
invoice-unmark-paid-dialog-body = La fattura tornerà non pagata. Utile per correggere un errore. Continuare?
invoice-unmark-paid-confirm = Annulla
invoice-unmark-paid-success = Contrassegno pagamento annullato
invoice-detail-paid-at-label = Pagata il

invoice-error-paid-at-required = Data di pagamento obbligatoria
invoice-error-paid-at-future = La data di pagamento non può essere futura
invoice-error-paid-at-before-invoice-date = La data di pagamento non può essere anteriore alla data fattura

echeancier-csv-header-number = Numero
echeancier-csv-header-date = Data
echeancier-csv-header-due-date = Scadenza
echeancier-csv-header-contact = Cliente
echeancier-csv-header-total = Totale
echeancier-csv-header-payment-status = Stato pagamento
echeancier-csv-header-paid-at = Data pagamento
echeancier-export-error-too-large = Troppi risultati (> { $limit }). Affinare i filtri (intervallo date o stato di pagamento) prima di esportare.
invoice-pdf-error-contact-missing = Il contatto collegato alla fattura non è stato trovato.
invoice-pdf-error-no-primary-bank = Nessun conto bancario principale configurato per questa azienda.
invoice-pdf-error-company-address-empty = L'indirizzo dell'azienda è vuoto — compilarlo prima di generare un PDF.
invoice-pdf-error-client-address-required = L'indirizzo del cliente è obbligatorio per generare il PDF.
invoice-pdf-error-client-address-empty = L'indirizzo del cliente è vuoto — compilarlo prima di generare un PDF.

common-loading = Caricamento…
common-previous = Precedente
common-next = Successivo
common-cancel = Annulla
common-error = Errore imprevisto

nav-invoices-due-dates = Scadenziario

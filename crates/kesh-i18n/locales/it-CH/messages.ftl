# Kesh — Messaggi italiano (Svizzera)

# Errori di autenticazione
error-invalid-credentials = Credenziali non valide
error-unauthenticated = Non autenticato
error-invalid-refresh-token = Sessione scaduta
error-rate-limited = Troppi tentativi

# Errori di autorizzazione
error-forbidden = Accesso negato
error-api-key-read-only = Questa chiave API è in sola lettura (scope «read»). Sono consentite solo le richieste GET.
error-api-key-management-forbidden = La gestione delle chiavi API non è consentita tramite una chiave API. Utilizzare l'interfaccia web.
error-api-key-admin-forbidden = Le rotte di amministrazione non sono accessibili tramite una chiave API, indipendentemente dal ruolo del suo creatore. Utilizzare l'interfaccia web.
error-cannot-disable-self = Impossibile disattivare il proprio account
error-cannot-disable-last-admin = Impossibile disattivare l'ultimo amministratore

# Errori di risorsa
error-not-found = Risorsa non trovata
error-conflict = Risorsa già esistente
error-optimistic-lock = Conflitto di versione — la risorsa è stata modificata
error-foreign-key = Riferimento non valido
error-journal-entry-linked-to-invoice = Questa scrittura contabile è stata generata da una fattura convalidata e non può essere eliminata direttamente. Annullare prima la fattura interessata.
error-check-constraint = Valore non valido
error-illegal-state = Transizione di stato non consentita

# Errori di validazione
error-validation = Errore di validazione
error-email-invalid = Formato email non valido
error-username-empty = Il nome utente non può essere vuoto
error-username-too-long = Il nome utente non deve superare { $max } caratteri
error-username-contains-at = Il nome utente non può contenere il carattere "@"
error-email-template-unknown-variables = Il modello contiene variabili sconosciute

# Errori di sistema
error-internal = Errore interno
error-service-unavailable = Servizio temporaneamente non disponibile
db-unavailable-banner = Database temporaneamente non disponibile — nuovo tentativo automatico in corso

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
demo-reset-error = Errore durante la reimpostazione

# Navigation sidebar (Story 6.3 + v014-1)
nav-home = Home
nav-contacts = Contatti
nav-products = Catalogo
nav-invoices = Fatture
nav-supplier-invoices = Fatture fornitori
nav-payment-batches = Pagamenti fornitori
nav-invoicing-due-dates = Scadenze
nav-invoicing-reminders = Solleciti
nav-settings = Impostazioni
# Story v014-1 — restructuration sidebar
nav-quotidien = Quotidiano
nav-mensuel = Mensile
nav-administration = Amministrazione
nav-accounts = Piano dei conti
nav-fiscal-years = Esercizi contabili
nav-opening-balances = Saldi iniziali
nav-bank-accounts = Conti bancari
nav-bank-profiles = Profili bancari
nav-reconciliation-rules = Regole di assegnazione

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
onboarding-stub-name-notice = La tua azienda ha un nome provvisorio — completa i tuoi dati
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
homepage-reminders-count = { $n } fattura/e da sollecitare
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
# --- Story 14-3a: ruoli dei conti e registrabilità ---
account-field-role = Ruolo
account-role-none = Nessuno
account-role-receivable = Crediti verso clienti
account-role-default-revenue = Ricavo predefinito
account-role-payable = Debiti verso fornitori
account-role-vat-recoverable = Imposta precedente (IVA recuperabile)
account-role-vat-payable = IVA dovuta
account-role-vat-settlement = Rendiconto IVA
account-role-equity-capital = Capitale
account-role-equity-other = Altri fondi propri
account-role-retained-earnings = Utile/perdita riportato
account-role-current-year-result = Risultato dell'esercizio
account-role-archived-hint = Ruolo inattivo — questo conto è archiviato
account-field-postable = Registrabile
account-postable-no = Non registrabile
account-postable-hint = Un conto non registrabile non accetta la registrazione manuale
accounts-reactivate-aria = Riattivare il conto { $number }
accounts-reactivated = Conto { $number } riattivato
accounts-role-conflict = Il ruolo è già assegnato al conto { $number } — { $name }. Rimuovetelo prima da quel conto.
accounts-error-number-required = Il numero è obbligatorio.
accounts-error-name-required = Il nome è obbligatorio.
accounts-error-number-exists = Questo numero di conto esiste già.
accounts-error-stale = La pagina non è più aggiornata. Ricaricatela e riprovate.
accounts-created = Conto { $number } creato
accounts-updated = Conto { $number } modificato
accounts-archived = Conto { $number } archiviato
accounts-count = { $count } conti
accounts-show-archived = Mostrare i conti archiviati
# Story 14-3a — code review: etichette dei dialoghi ed errori di ruolo
accounts-create-description = Aggiungete un conto al piano dei conti.
accounts-edit-title = Modificare il conto { $number }
accounts-edit-description = Il numero non è modificabile dopo la creazione.
accounts-archive-title = Archiviare il conto { $number }?
accounts-archiving = Archiviazione…
account-field-parent-optional = Conto padre (facoltativo)
accounts-parent-none = Nessuno
accounts-parent-archived = Il conto padre { $number } è archiviato. Riattivatelo prima.
accounts-role-invalid-for-type = Il ruolo { $role } non può essere attribuito a un conto di tipo { $type }.
accounts-role-conflict-generic = Questo ruolo è appena stato attribuito a un altro conto. Ricaricate la pagina.
accounts-reactivate-without-role = Riattivare senza il ruolo
accounts-reactivate-without-role-description = Il ruolo di questo conto è stato ripreso da un altro conto. Potete riattivarlo senza il suo ruolo — resterà modificabile in seguito.
accounts-reactivating = Riattivazione…
common-empty = Nessun elemento trovato.
common-create = Creare
common-creating = Creazione…
common-saving = Salvataggio…


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
journal-entry-form-col-project = Progetto
journal-entry-project-archived = Progetto archiviato
journal-entry-project-none = — Nessuno
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

# Conto di ricavo per riga di fattura (Story 16-1a, #152)
invoice-line-account-subject-line = Riga { $line }
invoice-line-account-subject-default = il conto di ricavo predefinito della società
invoice-line-account-unknown = { $subject }: il conto selezionato non esiste o non appartiene a questa società
invoice-line-account-inactive = { $subject }: il conto { $number } è archiviato
invoice-line-account-not-revenue = { $subject }: il conto { $number } non è un conto di ricavo
invoice-line-account-not-postable = { $subject }: il conto { $number } non è imputabile — scegliete un altro conto
invoice-line-revenue-account-invalid = Conto di ricavo non valido — { $detail }
credit-note-revenue-account-archived = Impossibile emettere la nota di credito — { $detail }. Riattivate i conti interessati.
invoice-error-total-zero = Questa fattura ha un importo totale nullo: non può essere convalidata. Inserite almeno una riga con un prezzo unitario superiore a zero.
credit-note-error-total-zero = Questa fattura ha un importo totale nullo: non è possibile emettere alcuna nota di credito.

# Modifica & eliminazione scritture (Story 3.3)
journal-entry-open = Apri la scheda

# Assistente IVA acquisto (Story 18-1c)
vat-purchase-title = Assistente IVA acquisto
vat-purchase-config-required = Configurare il conto dell'imposta precedente in Impostazioni → Fatturazione per usare l'assistente.
vat-purchase-no-rates = Nessuna aliquota IVA configurata — vedere Impostazioni → Aliquote IVA.
vat-purchase-charge-account = Conto di costo
vat-purchase-ht = Importo IVA esclusa
vat-purchase-rate = Aliquota IVA
vat-purchase-rate-placeholder = Scegliere un'aliquota
vat-purchase-counterparty = Conto di contropartita
vat-purchase-same-account = Il conto di costo e la contropartita devono essere diversi.
vat-purchase-recoverable-conflict = Il conto di costo e la contropartita non possono essere il conto dell'imposta precedente.
vat-purchase-insert = Inserisci le righe
vat-purchase-description = Acquisto — IVA { $rate } % recuperabile
vat-purchase-description-exempt = Acquisto — senza IVA
vat-purchase-replace-title = Sostituire la bozza?
vat-purchase-replace-message = Sono già state inserite righe o una descrizione. Continuando si sovrascriverà la bozza attuale.
vat-purchase-replace-confirm = Sostituisci
# Categorie aliquote IVA (riutilizzate dall'assistente IVA acquisto, Story 18-1c)
vat-category-normal = Aliquota normale
vat-category-reduced = Aliquota ridotta
vat-category-special = Aliquota speciale (alloggio)
vat-category-exempt = Esente / 0 %
vat-category-custom = Personalizzata
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
journal-entries-filter-account = Conto
journal-entries-filter-account-all = Tutti
journal-entries-filter-reset = Reimposta
journal-entries-pagination-on = su
journal-entries-pagination-prev = Precedente
journal-entries-pagination-next = Successivo
journal-entries-pagination-page-size = Per pagina
journal-entries-sort-asc-indicator = ordinamento crescente
journal-entries-sort-desc-indicator = ordinamento decrescente
journal-entries-loading = Caricamento…

# Story 24-4a (#380) — lo storno di una scrittura.
journal-entries-reverse-action = Stornare
journal-entries-reverse-dialog-title = Stornare questa scrittura?
journal-entries-reverse-dialog-body = Kesh creerà una scrittura inversa in data odierna. La scrittura originale resta intatta: la correzione deve vedersi, non sparire.
journal-entries-reverse-confirm = Stornare
journal-entries-reverse-cancel = Annulla
journal-entries-reverse-success = Scrittura stornata
journal-entries-reverses-link = Storna la scrittura n. { $number }
journal-entries-reversed-by-link = Stornata dalla scrittura n. { $number }
journal-entries-reverse-blocked-is-a-reversal = Questa scrittura è essa stessa uno storno.
journal-entries-reverse-blocked-already-reversed = Questa scrittura è già stata stornata.
journal-entries-reverse-blocked-invoice = Questa scrittura appartiene a una fattura cliente: correggetela con una nota di credito.
journal-entries-reverse-blocked-credit-note = Questa scrittura appartiene a una nota di credito, che è già uno storno.
journal-entries-reverse-blocked-supplier-invoice = Questa scrittura appartiene a una fattura fornitore: annullate la fattura.
journal-entries-reverse-blocked-settlement = Questa scrittura è un pagamento di fattura: il suo annullamento arriverà con lo storno dei pagamenti.
journal-entries-reverse-blocked-bank-match = Questa scrittura è riconciliata con una transazione bancaria.
journal-entries-reverse-blocked-account-archived = Un conto di questa scrittura è stato archiviato: riattivatelo per poterla stornare.
journal-entries-reverse-account-archived = Impossibile stornare — conto/conti archiviato/i: { $detail }. Riattivate i conti interessati.
journal-entries-delete-blocked-reversed = Questa scrittura è stata stornata: non può più essere eliminata.
journal-entries-blocked-posted = Una scrittura contabilizzata non può più essere modificata. Per correggerla, stornatela: Kesh crea la scrittura inversa e conserva l'originale.

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
# Story 22-2b (#301) — sonde anti-duplicato durante la digitazione.
contact-duplicate-heading = Contatti già registrati che potrebbero corrispondere
contact-duplicate-others-count-one = e 1 altro
contact-duplicate-others-count = e altri { $count }
contact-duplicate-ide-active = Questo numero IDI appartiene già a { $name }.
contact-duplicate-ide-archived = Questo numero IDI appartiene già a { $name }, che è archiviato. Un IDI resta riservato anche dopo l'archiviazione: la registrazione sarà rifiutata.
contact-form-client-number = Numero cliente
contact-form-client-number-hint = Appare sul PDF della fattura, affinché il cliente possa abbinarla al proprio dossier fornitori.
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
contact-filter-search-placeholder = Cerca per nome, e-mail, numero cliente o IDI senza separatori…
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
contact-error-payment-terms-days-range = Il termine di pagamento deve essere un numero intero tra 0 e 365 giorni
contact-error-ide-duplicate = Esiste già un contatto con questo numero IDI
contact-error-client-number-duplicate = Esiste già un contatto con questo numero cliente
contact-error-not-found = Contatto non trovato
contact-error-archived-no-modify = Contatto archiviato — modifica o ulteriore archiviazione vietata
contact-conflict-title = Conflitto di versione
contact-conflict-body = Questo contatto è stato modificato altrove. Vuoi ricaricare la versione attuale?
error-ide-already-exists = Esiste già un contatto con questo numero IDI

# Story 4.2 — Condizioni di pagamento e catalogo prodotti
contact-form-payment-terms = Condizioni di pagamento
contact-form-payment-terms-placeholder = es. 30 giorni netti
contact-payment-terms-days-label = { $days ->
    [one] Pagabile entro { $days } giorno
   *[other] Pagabile entro { $days } giorni
}
contact-payment-terms-immediate-label = Pagabile a vista
contact-form-payment-terms-days = Termine di pagamento (giorni)
contact-form-payment-terms-days-hint = La scadenza delle fatture sarà precalcolata e il testo delle condizioni generato automaticamente.
contact-form-payment-terms-disabled-hint = Testo generato automaticamente dal termine di pagamento.
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
product-error-vat-loading = Caricamento delle aliquote IVA in corso, attendere…
product-error-vat-fetch-failed = Impossibile caricare le aliquote IVA. Controlla la connessione di rete e ricarica la pagina.
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
invoices-page-title = Fatture
invoices-settings-vat-accounts-title = Conti IVA
invoices-settings-vat-accounts-hint = Conti usati per la contabilizzazione dell'IVA (predisposto per il rendiconto IVA dell'AFC).
invoices-settings-vat-payable = Conto IVA dovuta (Passivo)
invoices-settings-vat-recoverable = Conto IVA recuperabile (Attivo)
invoices-settings-vat-decompte = Conto rendiconto IVA (Passivo)
invoices-description-error-empty = Il testo di registrazione è vuoto
invoices-description-error-too-long = Il testo di registrazione supera { $max } caratteri (attuale: { $actual })
invoices-description-error-control-char = Carattere di controllo non consentito
invoices-description-error-unknown-placeholder = Segnaposto sconosciuto: {"{"}{ $name }{"}"}
invoices-description-error-no-placeholder = Il testo di registrazione deve contenere almeno un segnaposto riconosciuto ({"{"}YEAR{"}"}, {"{"}INVOICE_NUMBER{"}"}, {"{"}CONTACT_NAME{"}"})
invoices-format-error-empty = Il formato di numerazione è vuoto
invoices-format-error-too-long = Il formato supera { $max } caratteri (attuale: { $actual })
invoices-format-error-bad-chars = Il formato contiene caratteri non consentiti
invoices-format-error-bad-padding = Padding {"{"}SEQ:{ $n }{"}"} non valido — deve essere tra 1 e { $max }
invoices-format-error-unknown-placeholder = Segnaposto sconosciuto: {"{"}{ $name }{"}"}
invoices-format-error-no-placeholder = Il formato deve contenere almeno un segnaposto riconosciuto ({"{"}YEAR{"}"}, {"{"}FY{"}"}, {"{"}SEQ{"}"}, {"{"}SEQ:NN{"}"})
invoices-format-error-rendered-too-long = Il formato genererebbe un numero di { $len } caratteri (max { $max })
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
# Story 21-6a (D10) — suspension des rappels : badge + filtre en liste factures.
invoice-paused-badge = Sospeso
invoice-paused-filter-label = Solleciti
invoice-paused-filter-all = Tutti
invoice-paused-filter-paused = Sospesi
invoice-paused-filter-not-paused = Attivi
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
invoice-pdf-origin-reference = Rif. fattura originale
credit-note-pdf-title = Nota di credito
credit-note-pdf-number = N° nota di credito
invoice-pdf-ide = IDI
# Story 16-3a (#151) — coordonnées de contact de l'émetteur sur le PDF.
# ⚠️ Ces trois clés vivent AUSSI dans `I18N_KEYS`/`DEFAULT_EN` (kesh-qrbill).
invoice-pdf-phone = Tel.
invoice-pdf-email = E-mail
invoice-pdf-website = Web
invoice-pdf-client-number = N. cliente
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
error-invoice-too-many-lines-for-pdf = La fattura contiene { $count } righe — il PDF A4 monopagina non può mostrarle tutte con il riepilogo IVA. Riduca il numero di righe o divida la fattura.
error-pdf-generation-failed = Generazione PDF fallita.
# Story 9-2a + Pass 1 code-review H1 — variante CSV dedicata.
error-csv-generation-failed = Generazione CSV fallita.

# Story 5.4 — Scadenziario fatture
due-dates-title = Scadenziario
due-dates-link-aged = Vedi scadenziario per età
due-dates-link-reminders = Vedi i solleciti
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
payment-status-partial = Parzialmente pagata
invoice-amount-settled = Già pagato
invoice-amount-due = Saldo dovuto
invoice-settle-button = Registrare un pagamento
invoice-settle-button-short = Pagare
invoice-settle-dialog-title = Registrare un pagamento
invoice-settle-dialog-body = Il pagamento genera la sua registrazione contabile, qualunque sia la modalità.
invoice-settle-type-label = Modalità di pagamento
invoice-settle-type-bank = Bonifico bancario
invoice-settle-type-internal = Contanti o altro conto
invoice-settle-bank-label = Conto bancario
invoice-settle-account-label = Conto
invoice-settle-account-placeholder = — Scegliere un conto
invoice-settle-amount-label = Importo
invoice-settle-date-label = Data del pagamento
invoice-settle-confirm = Registrare il pagamento
invoice-settle-success-full = Pagamento registrato — fattura saldata
invoice-settle-success-partial = Pagamento parziale registrato
invoice-error-settled-on-required = Data del pagamento obbligatoria
invoice-error-settled-on-before-invoice-date = La data del pagamento non può precedere la data della fattura
invoice-error-amount-positive = L'importo deve essere superiore a zero
invoice-error-amount-over-due = L'importo supera il saldo dovuto su questa fattura
invoice-error-bank-account-required = Scegliete un conto bancario
invoice-error-account-required = Scegliete un conto

invoice-detail-paid-at-label = Pagata il

invoice-error-mark-paid-not-validated = Solo le fatture convalidate possono essere contrassegnate come pagate
invoice-error-already-unpaid = Questa fattura non è contrassegnata come pagata

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
common-back = Indietro
common-error = Errore imprevisto

invoice-pdf-error-not-found = Fattura non trovata.
invoice-pdf-error-generic = Errore durante il download del PDF.
invoice-pdf-error-empty = Il PDF ricevuto è vuoto.

# Story 2.6 — Onboarding: Invoice Settings Pre-fill
config-incomplete-title = Configurazione incompleta
config-incomplete-link = Configurare i conti di fatturazione
invoice-settings-required = Configurare innanzitutto i conti di fatturazione nelle impostazioni

# === Story 3.7 — Gestione esercizi contabili (IT-CH) ===

fiscal-year-title = Esercizi contabili
fiscal-year-list-empty = Nessun esercizio contabile.
fiscal-year-create-button = Nuovo esercizio
fiscal-year-name-label = Nome
fiscal-year-start-date-label = Data di inizio
fiscal-year-end-date-label = Data di fine
fiscal-year-status-label = Stato
fiscal-year-status-open = Aperto
fiscal-year-status-closed = Chiuso
fiscal-year-rename-button = Rinomina
fiscal-year-close-button = Chiudi
fiscal-year-close-confirmation-title = Chiudere questo esercizio?
fiscal-year-close-confirmation-body = Stai per chiudere l’esercizio « { $name } ». Finché resta chiuso, nessuna registrazione, fattura o pagamento potrà più essere registrato su questo periodo; solo un amministratore può riaprirlo (con un motivo tracciato). Confermare?
fiscal-year-close-confirmation-action = Chiudi
fiscal-year-created = Esercizio creato con successo.
fiscal-year-renamed = Esercizio rinominato.
fiscal-year-closed = Esercizio chiuso.
# Story 14-2 — riapertura di un esercizio chiuso (Admin, motivo, audit, regola LIFO)
fiscal-year-reopen-button = Riapri
fiscal-year-reopen-confirmation-title = Riaprire questo esercizio?
fiscal-year-reopen-confirmation-body = Stai per riaprire l’esercizio « { $name } ». Tornerà modificabile (registrazione di scritture) fino a una nuova chiusura. Un motivo è obbligatorio e sarà conservato nella pista di audit.
fiscal-year-reopen-motif-label = Motivo della riapertura
fiscal-year-reopen-confirmation-action = Riapri l’esercizio
fiscal-year-reopened = Esercizio riaperto.
fiscal-year-reopen-blocked-later-closed = Riapri prima l’esercizio « { $name } », più recente e ancora chiuso.
error-fiscal-year-reopen-motif-empty = Il motivo della riapertura è obbligatorio.
error-fiscal-year-reopen-motif-too-long = Il motivo della riapertura è troppo lungo (massimo 500 caratteri).
error-fiscal-year-already-open = Questo esercizio è già aperto.
error-fiscal-year-reopen-blocked = Riapertura impossibile: un esercizio successivo è chiuso; riaprilo prima.
error-fiscal-year-overlap = Questo esercizio si sovrappone a un esercizio esistente.
error-fiscal-year-name-duplicate = Un esercizio con questo nome esiste già.
error-fiscal-year-name-empty = Il nome dell’esercizio è obbligatorio.
error-fiscal-year-name-too-long = Il nome dell’esercizio è troppo lungo (massimo 50 caratteri).
error-fiscal-year-dates-invalid = Date non valide — la data di fine deve essere strettamente successiva alla data di inizio.
error-fiscal-year-already-closed = Questo esercizio è già chiuso.
error-fiscal-year-conflict = Conflitto sull’esercizio (nome o data di inizio già utilizzati).
error-fiscal-year-missing = Crea prima un esercizio contabile in Impostazioni → Esercizi.
error-fiscal-year-closed-for-date = L’esercizio che copre questa data è chiuso. Verifica la data inserita o consulta i tuoi esercizi.
go-to-settings = Apri impostazioni
settings-fiscal-years-link = Crea, rinomina o chiudi gli esercizi contabili della tua azienda.

# Story 14-4 — bilancio di apertura (saldi iniziali, ripresa della contabilità)
opening-balances-title = Saldi iniziali
opening-balances-intro = Inserisci i saldi dei tuoi conti di bilancio ripresi dalla contabilità precedente. Una registrazione di apertura equilibrata sarà generata al { $date } (primo giorno dell’esercizio « { $name } »). Registra il riporto a nuovo accumulato sul tuo conto di riporto per equilibrare la registrazione.
opening-balances-account = Conto
opening-balances-debit = Dare
opening-balances-credit = Avere
opening-balances-total-debit = Totale dare
opening-balances-total-credit = Totale avere
opening-balances-diff = Differenza
opening-balances-generate = Genera la registrazione di apertura
opening-balances-generating = Generazione…
opening-balances-success = Registrazione di apertura generata.
opening-balances-entry-description = Bilancio di apertura — saldi iniziali
opening-balances-locked-no-fiscal-year = Nessun esercizio contabile: crea prima un esercizio (Impostazioni → Esercizi) per inserire i saldi iniziali.
opening-balances-locked-first-year-closed = Il primo esercizio « { $name } » è chiuso: un amministratore deve riaprirlo prima dell’inserimento dei saldi iniziali.
opening-balances-locked-already-has-entries = L’azienda contiene già delle registrazioni: il bilancio di apertura è bloccato. Correggi la registrazione di apertura direttamente nel giornale, oppure elimina tutte le registrazioni per ricominciare.
opening-balances-goto-journal = Apri il giornale
opening-balances-goto-balance-sheet = Vedi il bilancio
opening-balances-status-error = Impossibile caricare lo stato dei saldi iniziali.
opening-balances-retry = Riprova
opening-balances-empty-grid = Nessun conto di bilancio attivo e registrabile nel piano dei conti — crea o riattiva prima i tuoi conti attivi e passivi (Piano dei conti) prima di inserire i saldi iniziali.
error-opening-balances-no-fiscal-year = Nessun esercizio contabile: crea prima un esercizio prima di inserire i saldi iniziali.
error-opening-balances-first-year-closed = Il primo esercizio è chiuso: riaprilo prima di inserire i saldi iniziali.
error-opening-balances-already-has-entries = L’azienda contiene già delle registrazioni: il bilancio di apertura non può più essere generato. Correggi la registrazione di apertura tramite il giornale.
error-opening-balances-non-balance-account = Il bilancio di apertura può toccare solo conti di bilancio (attivi e passivi) — rimuovi i conti di ricavo e di costo.


# --- Story 8-1b — Importazione bancaria CAMT.053 ---
bank-import-errors-too-large = File troppo grande. Dimensione massima consentita: 10 MiB.
bank-import-errors-malformed-xml = File XML malformato o troncato. Verificare l'esportazione bancaria.
bank-import-errors-unsupported-version = Versione CAMT.053 non supportata. Versioni accettate: 001.04 e 001.08.
bank-import-errors-missing-field = Un campo obbligatorio è assente nel file CAMT.053.
bank-import-errors-invalid-amount = Un importo nel file non è valido.
bank-import-errors-invalid-date = Una data nel file non è valida.
bank-import-errors-balance-mismatch = Il saldo finale non corrisponde alla somma delle transazioni. Spuntare «Conferma comunque» per importare.
bank-import-errors-unsupported-currency = Valuta non supportata. In questa versione è accettato solo il franco svizzero (CHF).
bank-import-errors-no-matching-statement = Nessun estratto del file corrisponde al conto bancario selezionato.
bank-import-errors-duplicate-file = Questo file è già stato importato per questa azienda.
bank-import-errors-bank-account-not-found = Conto bancario non trovato.
bank-import-errors-parse-failed = Il file CAMT.053 non ha potuto essere analizzato.

bank-import-warnings-balance-mismatch = Saldo finale incoerente.
bank-import-warnings-unsupported-currency = Valuta non supportata in v0.1.
bank-import-warnings-ignored-statements = Alcuni estratti del file non corrispondono al conto selezionato e saranno ignorati.
# Story 8-3 — rilevamento duplicati + accettazione parziale
bank-import-warnings-duplicate-file = Questo file è già stato importato.
bank-import-warnings-duplicate-lines-summary = transazioni si sovrappongono a un import precedente.
bank-import-warnings-invalid-lines-summary = righe non valide rilevate nel CSV.
bank-import-warnings-invalid-lines-truncated = Mostrati i primi 100 errori (limite raggiunto).
bank-import-warnings-encoding-mismatch = La codifica rilevata differisce dal profilo.

bank-import-labels-page-title = Importazione bancaria CAMT.053
bank-import-labels-bank-account-selector = Conto bancario di destinazione
bank-import-labels-drop-zone = Trascinare il file CAMT.053 qui o fare clic per sfogliare
bank-import-labels-preview-title = Anteprima
bank-import-labels-confirm-import = Conferma importazione
bank-import-labels-cancel = Annulla
bank-import-labels-confirm-balance-mismatch = Importa nonostante lo scarto di saldo
# Story 8-3 — flag di conferma + KF #70
bank-import-labels-confirm-duplicate-file = Importa nonostante il file già importato
bank-import-labels-confirm-duplicate-lines = Comportamento sui duplicati
bank-import-labels-confirm-duplicate-lines-skip = Ignora i duplicati (predefinito)
bank-import-labels-confirm-duplicate-lines-import = Importa comunque
bank-import-labels-confirm-partial-import = Importa comunque le righe valide
bank-import-labels-confirm-encoding-mismatch = Importa con la codifica rilevata
bank-import-labels-bank-profile-selector = Profilo bancario CSV
bank-import-labels-bank-profile-auto-matched = rilevato automaticamente
# L6 / M8 (Pass 1 review)
bank-import-labels-bank-profile-auto-detect-placeholder = Rilevamento automatico
bank-import-info-bank-csv-profile-auto-matched = Profilo bancario rilevato automaticamente dal nome del file.
bank-import-info-bank-csv-multiple-profile-matches = Più profili corrispondono al nome del file ; è stato selezionato il primo.
bank-import-errors-no-valid-lines-to-commit = Nessuna riga valida da importare nel CSV.
bank-import-labels-list-title = Importazioni precedenti
bank-import-labels-import-success = Importazione riuscita.
bank-import-labels-empty = Nessuna importazione bancaria.

# Story 8-2 — bank-csv + bank-profile keys
bank-import-csv-errors-no-profile-match = Nessun profilo bancario corrisponde a questo file.
bank-import-csv-errors-unsupported-encoding = Codifica del file non supportata (UTF-8 o ISO-8859-1 attesa).
bank-import-csv-errors-encoding-mismatch = La codifica rilevata differisce dal profilo. Conferma tramite confirmEncodingMismatch=true.
bank-import-csv-errors-partial-failure = Alcune righe del CSV non sono state elaborate.
bank-import-csv-errors-profile-invalid = Profilo bancario non valido.
bank-import-csv-errors-profile-duplicate = Un profilo con questo nome banca esiste già.
bank-import-csv-errors-profile-misconfigured = Profilo bancario configurato male.
bank-import-csv-errors-empty-file = File CSV vuoto o nessuna riga di dati.
bank-import-csv-errors-invalid-date = Data non valida in una riga CSV.
bank-import-csv-errors-invalid-amount = Importo non valido in una riga CSV.
bank-import-csv-errors-ambiguous-debit-credit = Dare e Avere entrambi non vuoti sulla stessa riga.
bank-import-csv-errors-empty-mandatory-field = Campo obbligatorio vuoto.
bank-import-csv-errors-row-too-short = Riga troppo corta (colonne mancanti).
bank-import-csv-warnings-profile-auto-matched = Profilo applicato automaticamente da auto-match.
bank-import-csv-warnings-multiple-profile-matches = Più profili corrispondono a questo nome file, è stato usato il più recente.
bank-import-csv-warnings-encoding-mismatch = La codifica rilevata differisce dal profilo.
bank-import-errors-unsupported-format = Formato file non supportato (CAMT.053 XML o CSV attesi).
bank-import-profile-labels-page-title = Profili bancari CSV
bank-import-profile-labels-page-title-new = Nuovo profilo bancario
bank-import-profile-labels-page-title-edit = Modifica profilo bancario
bank-import-profile-labels-bank-name = Nome della banca
bank-import-profile-labels-filename-pattern = Pattern nome file (regex)
bank-import-profile-labels-filename-pattern-help = Regex case-sensitive (usa `(?i)` per case-insensitive)
bank-import-profile-labels-date-format = Formato data (chrono)
bank-import-profile-labels-decimal-separator = Separatore decimale
bank-import-profile-labels-field-separator = Separatore campi
bank-import-profile-labels-encoding = Codifica (opzionale)
bank-import-profile-labels-actions = Azioni
bank-import-profile-labels-header-row-count = N. righe header (0-5)
bank-import-profile-labels-column-mapping = Mapping colonne (0-indicizzato)
bank-import-profile-labels-use-debit-credit-split = Colonne Dare/Avere separate
bank-import-profile-labels-create = Crea
bank-import-profile-labels-update = Aggiorna
bank-import-profile-labels-edit = Modifica
bank-import-profile-labels-delete = Elimina
bank-import-profile-labels-confirm-delete = Eliminare questo profilo?
bank-import-profile-labels-new-profile = Nuovo profilo
bank-import-profile-labels-no-profiles = Nessun profilo bancario configurato.
bank-import-profile-errors-bank-name-required = Il nome della banca è obbligatorio.
bank-import-profile-errors-bank-name-duplicate = Un profilo con questo nome esiste già.
bank-import-profile-errors-column-mapping-xor-violation = Scegli `amount` O `debit_credit_split`, non entrambi.
bank-import-profile-errors-date-format-invalid = Formato data chrono non valido.
bank-import-profile-errors-regex-invalid = Regex filename_pattern non valida.
bank-import-profile-errors-separators-equal = I separatori campi e decimale devono essere diversi.

# Story 8-4 (FR44) — Riconciliazione bancaria automatica.
reconciliation-page-title = Riconciliazione
reconciliation-page-subtitle = Proposte automatiche di matching transazione ↔ fattura.
reconciliation-labels-loading = Caricamento delle proposte…
reconciliation-labels-empty = Nessuna transazione in attesa di riconciliazione.
reconciliation-labels-no-account = Nessun conto bancario configurato.
reconciliation-labels-account-select = Conto bancario
reconciliation-labels-no-candidate = Nessuna corrispondenza
reconciliation-labels-success-suffix = operazione/i riuscita/e.
reconciliation-labels-failed = Errori parziali
reconciliation-cols-tx-date = Data
reconciliation-cols-tx-amount = Importo
reconciliation-cols-tx-counterparty = Controparte
reconciliation-cols-candidate = Candidato
reconciliation-cols-score = Score
reconciliation-actions-accept = Accettare
reconciliation-actions-reject = Rifiutare
# H8 Pass 1 code review — 8 chiavi canoniche AC #61.
reconciliation-labels-validate-selected = Convalida selezione
reconciliation-labels-reject-selected = Rifiuta selezione
reconciliation-labels-score = Punteggio
reconciliation-errors-account-locked = Conto bancario in fase di riconciliazione da parte di un altro utente. Riprova tra qualche secondo.
reconciliation-errors-already-reconciled = Questa transazione è già stata riconciliata.
reconciliation-errors-invoice-not-eligible = Questa fattura non è idonea alla riconciliazione.
reconciliation-toast-accept-success = { $count } transazione/i riconciliata/e con successo.
reconciliation-toast-reject-success = { $count } transazione/i rifiutata/e con successo.

# Story 8-5a-base FR45 — Riconciliazione manuale.
reconciliation-manual-button-label = Assegna manualmente
reconciliation-manual-modal-title = Riconciliazione manuale
reconciliation-manual-counterparty-label = Conto controparte
reconciliation-manual-description-label = Descrizione
reconciliation-manual-bank-account-not-configured = Il conto bancario non è configurato. Configurare il conto contabile collegato in /bank-accounts.
reconciliation-manual-value-date-label = Data valuta
reconciliation-manual-submit = Assegna
reconciliation-manual-error-no-proposal = Nessuna transazione selezionata
reconciliation-manual-error-counterparty-required = Conto controparte obbligatorio
reconciliation-manual-error-description-too-long = Descrizione troppo lunga (max { $max } caratteri)
reconciliation-manual-description-placeholder = Spese bancarie maggio

# Story 8-5a-bis FR48 — suddivisione di una transazione aggregata.
reconciliation-split-button-label = Suddividi
reconciliation-split-modal-title = Suddividi la transazione
reconciliation-split-balance-indicator = Saldo
reconciliation-split-error-imbalance = La suddivisione non bilancia l'importo della transazione.

reconciliation-cols-actions = Azioni

# Story 8-5a-zero — Collegamento `bank_account.journal_account_id`.
bank-accounts-labels-page-title = Conti bancari
bank-accounts-labels-page-subtitle = Collegare ogni conto bancario a un conto del piano dei conti (classe 1 tipica: 1020 Cassa, 1030 Banca).
bank-accounts-labels-bank-name = Banca
bank-accounts-labels-iban = IBAN
bank-accounts-labels-journal-account-id = Conto contabile collegato
bank-accounts-labels-not-configured = Non configurato
bank-accounts-labels-empty = Nessun conto bancario configurato.
bank-accounts-labels-loading = Caricamento…
bank-accounts-actions-link-account = Collega al piano dei conti
bank-accounts-actions-unlink-account = Scollega
bank-accounts-actions-cancel = Annulla
bank-accounts-actions-submit = Collega
bank-accounts-errors-account-not-found = Conto contabile non trovato.
bank-accounts-errors-invalid-account-type = Tipo di conto non valido (Attivo o Passivo richiesto).
# Story v014-1 — CRUD bank_accounts post-onboarding
bank-accounts-errors-has-transactions = Il conto bancario contiene transazioni — archiviazione rifiutata per preservare l'audit contabile.
bank-accounts-errors-cannot-archive-primary = Il conto principale non può essere archiviato finché esiste un altro conto non archiviato. Definire prima un altro conto come principale, poi archiviare questo.
bank-accounts-errors-onboarding-not-complete = L'onboarding deve essere completato (passo 7) prima di poter gestire i conti bancari.
# Story v014-1 — CRUD UI labels & actions (F3 Pass 1 code review parity DE/IT/EN)
bank-accounts-actions-create = Nuovo conto bancario
bank-accounts-actions-edit = Modifica
bank-accounts-actions-archive = Archivia
bank-accounts-actions-confirm-archive = Archivia
bank-accounts-actions-show-archived = Mostra archiviati
bank-accounts-actions-hide-archived = Nascondi archiviati
bank-accounts-actions-submit-create = Crea
bank-accounts-actions-submit-update = Salva
bank-accounts-labels-balance = Saldo
bank-accounts-labels-balance-unavailable = Saldo non disponibile (collegare al piano dei conti)
bank-accounts-labels-qr-iban = QR-IBAN (opzionale)
bank-accounts-error-qr-iban-not-qr = Questo IBAN non è un QR-IBAN. Se la vostra banca non vi ha fornito un QR-IBAN dedicato alle fatture QR, lasciate vuoto questo campo: il vostro IBAN normale è sufficiente.
bank-accounts-help-qr-iban = Da compilare solo se la vostra banca vi ha fornito un QR-IBAN dedicato alle fatture QR (numero speciale con un identificatore 30000–31999). Altrimenti lasciate vuoto questo campo: il vostro IBAN normale è sufficiente per generare fatture QR.
bank-accounts-labels-is-primary = Conto principale
bank-accounts-labels-primary-badge = Principale
bank-accounts-labels-archived-badge = Archiviato
bank-accounts-confirm-archive = Confermare l'archiviazione di questo conto bancario? Questa azione è irreversibile in v0.1.
bank-accounts-tooltip-journal-account = Collega questo conto bancario a un conto del piano dei conti (tipicamente 1020 Cassa, 1030 Banca). Permette alla riconciliazione automatica di creare le scritture sul conto corretto, e la visualizzazione del saldo sulla home page. Più conti: se ne hai diversi distinti (BCV + PostFinance), collega a un sotto-conto specifico (1030.001 BCV CHF), non al conto padre 1030.
bank-accounts-toast-create-success = Conto bancario creato.
bank-accounts-toast-update-success = Conto bancario modificato.
bank-accounts-toast-archive-success = Conto bancario archiviato.
# Story v014-1 — Homepage widget bank accounts (F14)
homepage-bank-total-liquidity = Totale liquidità
homepage-bank-total-partial = (solo conti collegati)
homepage-bank-balance-unavailable = Saldo non disponibile — collegare al piano dei conti
homepage-bank-last-transaction = Ultima transazione
settings-bank-manage = Gestisci in Amministrazione → Conti bancari
settings-bank-manage-hint = Per aggiungere, modificare o archiviare un conto bancario, utilizza la pagina dedicata Amministrazione → Conti bancari.
bank-accounts-toast-link-success = Conto bancario collegato con successo al piano dei conti.
bank-accounts-toast-unlink-success = Conto bancario scollegato dal piano dei conti.

# Story 8-5b — FR47 reconciliation rules. Traduzione IT da completare v0.2 (L51).
reconciliation-rules-page-title = Regole di assegnazione
reconciliation-rules-loading = Caricamento…
reconciliation-rules-labels-empty = Nessuna regola configurata.
reconciliation-rules-labels-label = Etichetta
reconciliation-rules-labels-match-type = Tipo
reconciliation-rules-labels-match-value = Valore
reconciliation-rules-labels-counterparty-account = Conto contropartita
reconciliation-rules-labels-priority = Priorità
reconciliation-rules-labels-priority-hint = Valore più basso = priorità più alta (1-1000)
reconciliation-rules-labels-applied-count = Applicata
reconciliation-rules-labels-status = Stato
reconciliation-rules-labels-active = Attiva
reconciliation-rules-labels-archived = Archiviata
reconciliation-rules-match-type-counterparty-contains = Controparte contiene
reconciliation-rules-match-type-counterparty-exact = Controparte esatta
reconciliation-rules-match-type-reference-contains = Riferimento contiene
reconciliation-rules-match-type-iban-exact = IBAN esatto
reconciliation-rules-form-title-create = Nuova regola
reconciliation-rules-form-title-edit = Modifica regola
reconciliation-rules-actions-new = Nuova regola
reconciliation-rules-actions-edit = Modifica
reconciliation-rules-actions-create = Crea
reconciliation-rules-actions-save = Salva
reconciliation-rules-actions-cancel = Annulla
reconciliation-rules-actions-archive = Archivia
reconciliation-rules-actions-deactivate = Disattiva
reconciliation-rules-actions-reactivate = Riattiva
reconciliation-rules-confirm-delete = Archiviare questa regola? Le scritture già applicate sono preservate.
reconciliation-rules-error-label-required = Etichetta richiesta
reconciliation-rules-error-match-value-required = Valore richiesto
reconciliation-rules-error-counterparty-required = Conto contropartita richiesto
reconciliation-rules-error-not-found = Regola non trovata.
reconciliation-rules-error-duplicate = Una regola attiva esiste già per questa combinazione tipo/valore.
reconciliation-rules-applied-badge = Regola
reconciliation-rules-applied-score-na = Auto
reconciliation-manual-project-label = Progetto analitico (facoltativo)
reconciliation-manual-project-none = — Nessuno
reconciliation-rules-default-project-archived = Progetto archiviato
reconciliation-rules-default-project-none = — Nessuno
reconciliation-rules-labels-default-project = Progetto analitico predefinito
reconciliation-split-add-line = + Aggiungi una riga
reconciliation-split-bank-account-not-configured = Il conto bancario non è configurato. Configurare il conto contabile collegato in /bank-accounts.
reconciliation-split-error-no-proposal = Nessuna transazione selezionata
reconciliation-split-project-none = — Nessuno
reconciliation-split-remove-line = -
reconciliation-split-th-account = Conto
reconciliation-split-th-amount = Importo
reconciliation-split-th-description = Descrizione
reconciliation-split-th-project = Progetto
reconciliation-split-value-date-label = Data valuta
reconciliation-split-error-min-lines = Sono richieste almeno { $min } righe
reconciliation-split-error-max-lines = Massimo { $max } righe
reconciliation-split-error-account-required = Riga { $line }: conto obbligatorio
reconciliation-split-error-amount-positive = Riga { $line }: importo > 0 obbligatorio
reconciliation-split-error-description-too-long = Riga { $line }: descrizione troppo lunga (max { $max })

# === Story 9-1 — Rapporti contabili (34 chiavi) ===
# TODO official translation — basico (Pass 1 ECH-19)

reports-balance-sheet = Bilancio
reports-income-statement = Conto economico
reports-trial-balance = Bilancio di verifica
reports-journals = Giornali

reports-column-account-number = N° conto
reports-column-account-name = Denominazione
reports-column-debit = Dare
reports-column-credit = Avere
reports-column-balance = Saldo
reports-column-entry-date = Data
reports-column-description = Causale

reports-section-assets = Attivi
reports-section-liabilities = Passivi
reports-section-equity = Patrimonio netto
reports-section-revenues = Ricavi
reports-section-expenses = Costi

reports-total-assets = Totale attivi
reports-total-liabilities = Totale passivi
reports-total-equity = Totale patrimonio netto
reports-total-revenues = Totale ricavi
reports-total-expenses = Totale costi
reports-total-debit = Totale dare
reports-total-credit = Totale avere
reports-net-result = Risultato netto
reports-grand-total = Totale generale
# Rapporto IVA (Story 11-2)
reports-vat = IVA
reports-vat-column-rate = Aliquota
reports-vat-column-base-ht = Cifra d'affari netta
reports-vat-column-vat-due = IVA dovuta
reports-vat-total-base-ht = Totale cifra d'affari netta
reports-vat-recoverable = IVA recuperabile
reports-vat-balance = Saldo
reports-vat-reconciliation-warning = Il rendiconto non corrisponde alle registrazioni contabili (scarto: { $delta }). Verificare le registrazioni convalidate modificate manualmente.

reports-filter-period = Periodo
reports-filter-fiscal-year = Esercizio
reports-filter-journal = Giornale
reports-button-generate = Genera

reports-error-no-entries-in-period = Nessuna registrazione nel periodo selezionato. Modifica le date o scegli un altro esercizio.
reports-error-period-out-of-fiscal-year = Il periodo selezionato supera i limiti dell'esercizio. Scegli un periodo tra { $fyStart } e { $fyEnd }.
reports-error-no-fiscal-year-available = Nessun esercizio contabile disponibile. Crea un esercizio prima di generare i rapporti.

reports-equity-result-section-title = Risultato del periodo (prima della chiusura contabile)
reports-equity-result-profit = Utile del periodo
reports-equity-result-loss = Perdita del periodo
reports-retained-earnings = Risultato riportato
reports-retained-earnings-calculated = Risultato riportato (calcolato)
reports-retained-earnings-loss = Perdita riportata
reports-trial-balance-period-note = Il bilancio di verifica mostra il movimento del periodo (per esercizio). Il totale per conto non è comparabile con il saldo cumulato dello stesso conto nel bilancio (riporto a nuovo dall'origine).

# Alerts + badges UI (2 — code review Pass 1 i18n leaks)
reports-equation-warning = ⚠️ Equazione di bilancio non bilanciata (verificare i dati sorgente).
reports-archived-label = archiviato

# Pagina rapporti — chrome (3 — code review Pass 1 i18n leaks)
reports-page-title = Rapporti contabili
reports-instruction-select-and-generate = Seleziona un esercizio contabile e clicca su Genera.
reports-loading = Generazione del rapporto in corso…

# Story 9-2a — Export PDF & CSV (10 chiavi)
reports-export-pdf-button = Esporta PDF
reports-export-csv-button = Esporta CSV
# Story 21-7 — Scadenziario per età
reports-aged-balance = Scadenziario per età
reports-fiscal-year-label = Esercizio contabile
reports-generate = Genera
reports-project-expenses = Spese per progetto
reports-project-expenses-col-account = Conto
reports-project-expenses-col-amount = Importo
reports-project-expenses-empty = Nessuna spesa associata a questo progetto per il periodo.
reports-project-expenses-subtotal = Subtotale
reports-project-expenses-title = Spese per progetto
reports-project-expenses-total = Totale spese
reports-project-mode-cumulative = Cumulato dall’origine
reports-project-mode-fiscal-year = Esercizio contabile
reports-project-mode-label = Periodo
reports-project-return = Rendimento per progetto
reports-project-return-col-cost = Costo investito
reports-project-return-col-net = Risultato netto
reports-project-return-col-project = Progetto
reports-project-return-col-return = Rendimento
reports-project-return-col-revenue = Ricavi
reports-project-return-empty = Nessun movimento associato a questo progetto per il periodo.
reports-project-return-title = Rendimento per progetto
reports-project-return-total = Totale
reports-project-selector-label = Progetto
reports-project-selector-placeholder = — Scegli un progetto
reports-aged-instruction = Scadenziario per età alla data odierna.
reports-aged-instruction-generate = Clicca su Genera per visualizzare lo scadenziario per età alla data odierna.
reports-aged-as-of = Al { $date }
reports-aged-empty = Nessun credito cliente aperto.
reports-aged-col-contact = Cliente
reports-aged-col-not-due = Non scaduto
reports-aged-col-1-30 = 1-30 g
reports-aged-col-31-60 = 31-60 g
reports-aged-col-61-90 = 61-90 g
reports-aged-col-over-90 = 90+ g
reports-aged-col-total = Totale
reports-aged-total-row = Totale generale
reports-aged-link-due-dates = Vedi scadenziario
reports-export-loading = Generazione del file…
reports-export-error-generic = Impossibile esportare il rapporto. Verifica la connessione e riprova.
reports-filename-balance-sheet = bilancio
reports-filename-income-statement = conto-economico
reports-filename-trial-balance = bilancio-verifica
reports-filename-journals = giornali
reports-filename-vat = rendiconto-iva
reports-filename-project-expenses = spese-per-progetto
reports-filename-project-return = rendimento-per-progetto
reports-filename-general-ledger = libro-mastro
reports-pdf-header-period = Periodo
reports-pdf-empty-message = Nessuna registrazione nel periodo selezionato.

# Story 9-2b — Esportazione globale ZIP (sovranità dei dati) — 12 chiavi
nav-export-global = Esportazione globale
# Story 17-3b — backup completo dell'installazione (.keshbackup, Admin)
nav-admin-backup = Backup completo
admin-backup-page-title = Backup completo dell'installazione
admin-backup-page-description = Scarica l'intera installazione (tutte le società, gli utenti e i dati di sistema) in un unico file .keshbackup, per migrare o salvare. Da distinguere dall'esportazione globale di una singola società.
admin-backup-action-export = Esporta l'intera installazione
admin-backup-action-exporting = Esportazione in corso…
admin-backup-toast-success = Backup dell'installazione scaricato.
admin-backup-error-generic = Esportazione dell'installazione non riuscita. Riprovate tra poco.
admin-backup-page-hint-secret = Il file .keshbackup contiene dati sensibili (credenziali, token). Conservatelo in un luogo sicuro.
# Story 17-3d — import / ripristino completo dell'installazione (.keshbackup, Admin)
nav-admin-restore = Ripristina / Importa
admin-restore-page-title = Ripristino / import dell'installazione
admin-restore-page-description = Caricate un file .keshbackup per sostituire l'intera installazione attuale (migrazione o ripristino). Operazione distruttiva: prima dell'import viene creato un backup dello stato attuale lato server.
admin-restore-file-label = File .keshbackup da importare
admin-restore-action-import = Importa e sostituisci l'installazione
admin-restore-action-importing = Import in corso…
admin-restore-confirm-title = Sostituire l'intera installazione?
admin-restore-confirm-body = Questa azione sostituirà TUTTI i dati dell'installazione attuale. Prima dell'import verrà creato un backup dello stato attuale lato server. Sarete disconnessi e dovrete riconnettervi con le credenziali dell'istanza importata.
admin-restore-confirm-cancel = Annulla
admin-restore-confirm-ok = Conferma la sostituzione
admin-restore-toast-success = Import riuscito — verrete disconnessi.
admin-restore-error-version = Questo backup richiede una versione di Kesh più recente ({ $src }) di quella installata ({ $bin }). Aggiornate Kesh prima di reimportare.
admin-restore-error-schema = Schema del backup non compatibile con questa versione di Kesh (tabella { $table }).
admin-restore-error-invalid = File di backup non valido o danneggiato. Assicuratevi che sia un file .keshbackup prodotto da Kesh.
admin-restore-error-generic = Import non riuscito. Lo stato precedente dell'installazione è stato preservato.
export-global-title = Esportazione globale dei vostri dati
export-global-description = Esportate i vostri dati contabili (conti, scritture, contatti, prodotti, fatture di vendita, conti e transazioni bancarie) in formato CSV in un file ZIP. Utilizzate questa esportazione per archiviare o conservare i vostri dati per 10 anni (CO svizzero art. 958f). ⚠️ Non copre ancora l'insieme della vostra contabilità: leggete qui sotto cosa non contiene prima di farne affidamento per migrare verso un altro software.
export-global-button = Avvia esportazione
export-global-loading = Generazione esportazione…
export-global-success = Esportazione scaricata.
export-global-error-generic = Impossibile generare l'esportazione globale. Verificate la connessione e riprovate.
export-global-filename-hint = Il file sarà scaricato con il nome kesh-export-{ $companyShort }-{ $date }.zip
export-global-content-includes = L'esportazione contiene: piano dei conti, esercizi, scritture, contatti, prodotti, fatture, conti bancari, cronologia degli import bancari, transazioni, aliquote IVA attive e storiche, impostazioni di fatturazione, regole di riconciliazione, profili di import bancario, e un manifesto metadata.json con hash SHA-256 di ogni file per la verifica di integrità.
export-global-content-excludes = Non contiene: fatture fornitori e relative righe, note di credito, progetti analitici (le scritture riportano un identificativo di progetto, ma la tabella dei progetti è assente), lotti di pagamento, persone di contatto, giustificativi importati, utenti (dati personali e password), token di sessione, registro audit interno, stato di onboarding.
export-global-souverainete-note = I vostri dati vi appartengono. Kesh non effettua alcuna copia di questa esportazione sui suoi server.
error-global-export-failed = L'esportazione globale non è potuta essere generata. Se il problema persiste, contattate l'assistenza.
error-admin-full-export-failed = L'esportazione completa dell'installazione non è potuta essere generata. Riprovate tra poco; se il problema persiste, contattate l'assistenza.
error-admin-full-import-failed = L'importazione dell'installazione non è riuscita. Lo stato precedente è stato preservato (prima dell'operazione è stato creato un backup automatico). Controllate i log del server e riprovate.
error-invalid-backup-structure = Il file di backup non è valido o è danneggiato (struttura inattesa o controllo di integrità fallito). Assicuratevi che sia un file .keshbackup prodotto da Kesh.
error-import-schema-mismatch = Lo schema di questo backup non è compatibile con questa versione di Kesh. Aggiornate Kesh o utilizzate un backup compatibile.
error-import-client-number-collision = Il backup contiene numeri cliente in collisione (identici dopo la normalizzazione di maiuscole, accenti e caratteri invisibili). Correggete le schede indicate nel rapporto, riesportate e riprovate.
error-import-version-incompatible = Questo backup richiede una versione di Kesh più recente di quella installata. Aggiornate Kesh prima di reimportare.

# Story v011-5 — Onboarding self-service (12 chiavi UI + 2 chiavi errore)
error-setup-required = Configurazione iniziale richiesta. Creare l'account amministratore tramite /setup.
error-setup-already-complete = L'account amministratore è già stato creato.
setup-welcome = Benvenuti in Kesh
setup-intro = Per completare l'installazione, creare l'account amministratore iniziale. Questo account avrà i diritti completi sulla vostra istanza Kesh.
setup-username-label = Nome utente
setup-username-placeholder = admin
setup-username-required = Il nome utente è obbligatorio.
setup-password-label = Password
setup-password-min = Almeno 12 caratteri.
setup-password-confirm-label = Conferma password
setup-password-mismatch = Le password non corrispondono.
setup-email-label = Email (consigliata)
setup-email-hint = Consente di reimpostare la password via email in caso di dimenticanza.
setup-email-invalid = Formato email non valido.
setup-submit = Crea account amministratore
setup-error-already-complete = L'account amministratore è già stato creato. Sarete reindirizzati alla pagina di accesso.
setup-error-rate-limit = Troppi tentativi. Riprova tra qualche minuto.

# === Story 17-2b — Chiavi API (PAT) frontend (36 chiavi) ===
# Impostazioni → link
settings-api-keys-title = Chiavi API
settings-api-keys-manage = Gestisci
settings-api-keys-hint = Crea chiavi di accesso API per le tue integrazioni (IA esterna, script, software di terze parti).
# Pagina chiavi API — etichette
api-keys-labels-page-title = Chiavi API
api-keys-labels-page-subtitle = Crea chiavi di accesso API per le tue integrazioni (IA esterna, script, software di terze parti). Presenta la chiave tramite l'intestazione «Authorization: Bearer».
api-keys-labels-name = Nome
api-keys-labels-name-placeholder = es. Script contabile, Agente IA…
api-keys-labels-scope = Ambito
api-keys-labels-scope-read = Sola lettura
api-keys-labels-scope-read-write = Lettura-scrittura
api-keys-labels-expires = Scadenza (facoltativa)
api-keys-labels-expires-hint = Lascia vuoto per una chiave permanente.
api-keys-labels-created-at = Creata il
api-keys-labels-last-used = Ultimo utilizzo
api-keys-labels-never-used = Mai utilizzata
api-keys-labels-status = Stato
api-keys-labels-status-active = Attiva
api-keys-labels-status-expires = Attiva (scade il { $date })
api-keys-labels-status-revoked = Revocata il { $date }
api-keys-labels-status-expired = Scaduta il { $date }
api-keys-labels-empty = Nessuna chiave API. Creane una per le tue integrazioni.
api-keys-labels-loading = Caricamento…
api-keys-labels-secret-created = Chiave «{ $name }» creata.
api-keys-labels-secret-warning = Copia questa chiave ora: non sarà più visualizzata.
# Azioni
api-keys-actions-create = Nuova chiave
api-keys-actions-submit-create = Crea la chiave
api-keys-actions-cancel = Annulla
api-keys-actions-copy = Copia
api-keys-actions-close = Chiudi
api-keys-actions-revoke = Revoca
api-keys-actions-confirm-revoke = Revoca
# Conferma
api-keys-confirm-revoke = Revocare questa chiave? Ogni integrazione che la utilizza smetterà immediatamente di funzionare. Questa azione è irreversibile.
# Errori
api-keys-errors-name-required = Il nome della chiave è obbligatorio.
api-keys-errors-name-too-long = Il nome della chiave è troppo lungo (massimo 255 caratteri).
api-keys-errors-conflict = La chiave è cambiata nel frattempo — elenco ricaricato, riprova.
# Toast
api-keys-toast-create-success = Chiave API creata.
api-keys-toast-copied = Chiave copiata negli appunti.
api-keys-toast-copy-failed = Copia non riuscita — seleziona e copia manualmente.
api-keys-toast-revoke-success = Chiave revocata.

# Story 17-4b — Recupero della password via email (rendering backend, DC10)
error-smtp-send-failed = Invio dell'email non riuscito. Riprova tra qualche istante.

# Story 20-3b1 — invio fattura via e-mail
error-smtp-not-configured = L'invio di e-mail non è configurato su questa istanza (variabili KESH_SMTP_*).
error-contact-email-missing = Il contatto della fattura non ha un indirizzo e-mail. Inserirlo nella scheda contatto.
error-invoice-email-empty-content = L'oggetto e il corpo dell'e-mail non possono essere vuoti.
error-invoice-due-date-before-date = La scadenza non può essere anteriore alla data della fattura
error-contact-archived = Il contatto della fattura è archiviato. Riattivarlo prima di inviare la fattura per e-mail.
error-email-sent-invoice-gone = L'e-mail è stata inviata al contatto, ma la fattura è stata eliminata nel frattempo — non è stato possibile contrassegnarla come « inviata ». Non inviare di nuovo l'e-mail.
error-company-email-invalid = L'indirizzo e-mail dell'azienda non è valido.
error-invalid-or-expired-token = Link di reimpostazione non valido o scaduto.
email-password-reset-subject = Reimpostazione della password Kesh
email-password-reset-body =
    Hai richiesto la reimpostazione della tua password Kesh.
    Apri il link seguente per scegliere una nuova password (valido per { $ttlMinutes } minuti):
    { $resetUrl }
    Se non hai effettuato questa richiesta, ignora questa email.

# Story 17-4d — Recupero della password (pagine pubbliche frontend)
auth-recovery-forgot-title = Password dimenticata
auth-recovery-forgot-intro = Inserisci il tuo nome utente o il tuo indirizzo email. Se un account corrisponde, riceverai un link di reimpostazione.
auth-recovery-identifier-label = Nome utente o email
auth-recovery-submit = Invia il link di reimpostazione
auth-recovery-success-generic = Se un account corrisponde a questo identificativo, gli è appena stata inviata un'email con un link di reimpostazione. Il link è valido per 30 minuti.
auth-recovery-error-rate-limit = Troppi tentativi. Riprova tra qualche minuto.
auth-recovery-error-network = Impossibile contattare il server. Verifica la connessione.
auth-recovery-error-unavailable = La reimpostazione via email non è disponibile. Contatta il tuo amministratore.
auth-recovery-error-server = Errore del server. Riprova più tardi.
auth-recovery-back-to-login = Torna all'accesso
auth-recovery-reset-title = Nuova password
auth-recovery-reset-intro = Scegli la tua nuova password.
auth-recovery-new-password-label = Nuova password
auth-recovery-password-confirm-label = Conferma la password
auth-recovery-password-min = Almeno 12 caratteri.
auth-recovery-password-mismatch = Le password non corrispondono.
auth-recovery-reset-submit = Reimposta la password
auth-recovery-reset-success = La tua password è stata reimpostata. Ora puoi accedere.
auth-recovery-invalid-link = Questo link di reimpostazione non è valido o è scaduto. Fai una nuova richiesta per ricevere un nuovo link.
auth-recovery-request-new-link = Nuova richiesta
auth-recovery-login-cta = Accedi

# Story 12.2 — factures fournisseurs (#191)
supplier-invoices-title = Fatture fornitori

# Story 12.3 — paiements pain.001 (#191)
payment-batches-title = Pagamenti fornitori

# Story 20-3b2 — invio fattura per e-mail (UI)
common-save = Salva
common-admin-only = Accesso riservato agli amministratori.
common-edit = Modifica
error-unexpected = Errore imprevisto.
invoice-send-email-button = Invia per e-mail
invoice-resend-email-button = Invia di nuovo per e-mail
invoice-send-email-smtp-tooltip = L'invio di e-mail non è configurato (variabili KESH_SMTP_*) — vedere il manuale dell'amministratore.
invoice-send-email-title = Inviare la fattura per e-mail
invoice-send-email-to-label = Destinatario
invoice-send-email-to-missing = Il contatto non ha un indirizzo e-mail — inserirlo nella scheda contatto.
invoice-send-email-subject-label = Oggetto
invoice-send-email-body-label = Messaggio
invoice-send-email-confirm = Invia l'e-mail
invoice-send-email-success = Fattura inviata per e-mail
invoice-send-email-error-empty = L'oggetto e il corpo dell'e-mail non possono essere vuoti.
invoice-detail-emailed-at-label = Inviata il
contact-form-language = Lingua di corrispondenza
contact-form-language-inherited = Ereditata (lingua dell'istanza)
contact-form-salutation = Titolo
contact-salutation-neutre = Neutro
contact-salutation-monsieur = Signor
contact-salutation-madame = Signora
# ── Story 23-1b (#316) — dominio pilota « contacts », 20 chiavi.
# Registre : 2ᵉ personne du singulier (D9), registre majoritaire du fichier — 31 impératifs
# au singulier contre 11 messages de courtoisie. « NPA » est l'usage de la Poste suisse en
# italien, et non « CAP » qui est italien.
contact-error-address-npa-city = NPA e località sono obbligatori se viene inserito un indirizzo
contact-error-person-name = Nome e cognome sono obbligatori per una persona
contact-persons-add = Aggiungi
contact-persons-add-error = Impossibile aggiungere la persona
contact-persons-delete = Elimina
contact-persons-delete-error = Impossibile eliminare la persona
contact-persons-empty = Nessuna persona di contatto.
contact-persons-hint = a titolo informativo
contact-persons-load-error = Impossibile caricare le persone di contatto
contact-persons-name-required = Nome e cognome sono obbligatori
contact-persons-role = Funzione
contact-persons-title = Persone di contatto
field-address = Indirizzo
field-building = N.
field-city = Località
field-country = Paese
field-first-name = Nome
field-last-name = Cognome
field-postal-code = NPA
field-street = Via
settings-field-company-email = E-mail (indirizzo di risposta)
settings-company-email-help = Indirizzo di risposta (Reply-To) delle fatture inviate per e-mail. Vuoto = nessun indirizzo di risposta.
settings-company-email-invalid = Indirizzo e-mail non valido.
settings-company-email-saved = E-mail della società salvata
settings-company-email-conflict = Conflitto di versione — i dati sono stati ricaricati, riprovare.
settings-company-email-conflict-reload-failed = Conflitto di versione e ricaricamento impossibile — ricaricare la pagina.

# --- Solleciti debitori (Story 21-4, #231) ---
dunning-title = Solleciti debitori
dunning-subtitle = Configura i livelli di sollecito, le scadenze e le spese di richiamo.
dunning-load-error = Impossibile caricare le impostazioni di sollecito.
dunning-grace-heading = Periodo di tolleranza
dunning-grace-help = Giorni dopo la scadenza prima che il 1° sollecito diventi esigibile.
dunning-grace-label = Tolleranza (giorni)
dunning-grace-save = Salva
dunning-grace-saved = Periodo di tolleranza salvato.
dunning-levels-heading = Livelli di sollecito
dunning-level-new = Aggiungi un livello
dunning-empty = Nessun livello configurato — i solleciti sono disattivati.
dunning-col-level = Livello
dunning-col-delay = Scadenza (giorni)
dunning-col-fee = Spese (CHF)
dunning-col-actions = Azioni
dunning-edit = Modifica
dunning-delete = Elimina
dunning-example-heading = Scadenzario previsionale
dunning-example-line = { $level }° sollecito proposto { $days } g dopo la scadenza
dunning-cgv-hint = Le spese di sollecito sono esigibili solo con una base contrattuale (CGC). Non sono incluse nel QR della fattura allegata.
dunning-delay-label = Scadenza (giorni)
dunning-delay-help = Giorni dalla fase precedente (scadenza + tolleranza per il 1°).
dunning-fee-label = Spese (CHF)
dunning-form-submit = Salva
dunning-form-cancel = Annulla
dunning-form-error = Salvataggio non riuscito.
dunning-form-error-delay = La scadenza deve essere un numero intero positivo.
dunning-form-error-fee = Le spese devono essere un importo valido.
dunning-delete-confirm-body = Eliminare questo livello di sollecito? I livelli successivi saranno rinumerati.
dunning-delete-confirm-action = Elimina
dunning-created = Livello di sollecito aggiunto.
dunning-updated = Livello di sollecito aggiornato.
dunning-deleted = Livello di sollecito eliminato.
dunning-conflict = Il livello è stato modificato nel frattempo, ricaricato.
dunning-delete-error = Eliminazione non riuscita.
dunning-grace-error = Il periodo di tolleranza deve essere un numero intero positivo.
dunning-settings-conflict = Le impostazioni sono cambiate nel frattempo, ricaricate.
settings-dunning-link = Configura i livelli di sollecito (scadenze e spese), il periodo di tolleranza e personalizza i testi delle e-mail di sollecito per livello.
email-templates-type-invoice_send = Invio fattura
email-templates-type-invoice_reminder = Sollecito di pagamento
email-templates-level-generic = Generico
email-templates-level-n = Sollecito { $n }
email-templates-type-label = Tipo
email-templates-level-label = Livello

# Story 21-6b — Solleciti (invio dei solleciti ai debitori)
reminders-page-title = Solleciti
reminders-forbidden = Accesso riservato a contabili e amministratori.
reminders-empty = Nessuna fattura da sollecitare.
reminders-level-name = Sollecito { $level }
reminders-level-next = Prossimo: sollecito { $level }
reminders-last-sent = ultimo il { $date }
reminders-select-invoice = Seleziona { $inv }
reminders-selected-count = { $n } selezionata/e
reminders-batch-cap = Massimo { $cap } fatture per lotto.
reminders-batch-send = Invia i solleciti selezionati
reminders-sending = Invio…
reminders-saving = Salvataggio…
reminders-badge-no-email = senza e-mail
reminders-badge-terminal = Ultimo livello raggiunto
reminders-batch-accepted = { $n } sollecito/i inviato/i.
reminders-batch-failed = { $n } errore/i:
reminders-send-title = Invia un sollecito
reminders-send-open = Invia un sollecito
reminders-send-level-label = Livello di sollecito
reminders-send-to-label = Destinatario
reminders-send-no-recipient = Il contatto non ha un indirizzo e-mail.
reminders-send-subject-label = Oggetto
reminders-send-body-label = Messaggio
reminders-send-empty = L'oggetto e il corpo non possono essere vuoti.
reminders-send-confirm = Invia il sollecito
reminders-send-success = Sollecito inviato
reminders-manual-title = Registra un sollecito manuale
reminders-manual-open = Sollecito manuale
reminders-manual-body = Registra un sollecito già inviato fuori da Kesh (lettera, raccomandata). Nessuna e-mail verrà inviata.
reminders-manual-level-label = Livello di sollecito
reminders-manual-date-label = Data di invio
reminders-manual-date-required = Data di invio obbligatoria
reminders-manual-date-future = La data di invio non può essere nel futuro
reminders-manual-note-label = Nota (facoltativa)
reminders-manual-confirm = Registra
reminders-manual-success = Sollecito manuale registrato
reminders-error-invoice-not-found = Fattura non trovata
reminders-error-invoice-not-validated = Fattura non validata
reminders-error-invoice-already-paid = Fattura già pagata
reminders-error-dunning-paused = Solleciti sospesi
reminders-error-no-next-level = Ultimo livello raggiunto
reminders-error-contact-archived = Contatto archiviato
reminders-error-contact-email-missing = Contatto senza indirizzo e-mail
reminders-error-content-empty = Modello di sollecito vuoto
reminders-error-content-too-long = Contenuto del sollecito troppo lungo
reminders-error-not-pdf-ready = Fattura non stampabile in PDF
reminders-error-rate-limited = Limite di invio raggiunto
reminders-error-database-error = Errore tecnico
reminders-error-smtp-failed = Invio e-mail non riuscito
reminders-error-sent-but-gone = E-mail inviata, ma la fattura è scomparsa nel frattempo (non registrato)
reminders-error-sent-not-recorded = E-mail inviata, ma non registrata (errore tecnico)
reminders-error-unknown = Errore ({ $code })
# Story 21-6c — Cronologia e sospensione nella scheda fattura
reminders-history-title = Cronologia dei solleciti
reminders-history-empty = Nessun sollecito inviato.
reminders-history-col-date = Data
reminders-history-col-level = Livello
reminders-history-col-channel = Canale
reminders-history-col-recipient = Destinatario
reminders-history-col-fee = Spese
reminders-history-channel-email = E-mail
reminders-history-channel-manual = Manuale
reminders-history-cancelled-at = Annullato il { $date }
reminders-pause-button = Sospendere i solleciti
reminders-resume-button = Riprendere i solleciti
reminders-pause-title = Sospendere i solleciti
reminders-pause-body = I solleciti automatici di questa fattura sono sospesi fino alla ripresa. Puoi annotare il motivo (contestazione, accordo).
reminders-pause-note-label = Motivo (facoltativo)
reminders-pause-confirm = Sospendere
reminders-pause-submitting = Sospensione…
reminders-pause-success = Solleciti sospesi
reminders-resume-success = Solleciti ripresi
reminders-error-not-paused = Questa fattura non è più sospesa.
reminders-link-due-dates = Vedi scadenziario
reminders-link-aged = Vedi scadenziario per età

# Story 16-1b (#152) — selecteur de compte de produit par ligne.
common-account-clear = Cancella il conto selezionato
common-account-invalid = Conto non valido — non imputabile, archiviato o di tipo inatteso
invoice-line-col-revenue-account = Conto di ricavo
invoice-detail-project-label = Progetto analitico
invoice-field-project = Progetto analitico (facoltativo)
invoice-project-archived = Progetto archiviato
invoice-project-current = Progetto attuale
invoice-project-none = — Nessuno
invoice-line-revenue-account-default = { $account } (predefinito della società)
invoice-lines-revenue-account-invalid = Conto di ricavo non valido nelle righe seguenti: { $lines }
invoice-default-revenue-account-unusable = Il conto di ricavo predefinito della società non è più utilizzabile (archiviato, non imputabile o di tipo inatteso). Le righe che lo seguono non potranno essere validate — correggilo nelle Impostazioni oppure scegli un conto per ogni riga.
common-account-default-suffix = (predefinito)
invoice-line-aria-line = riga { $n }

# Story 16-2a (#144) — conto di ricavo sulla scheda articolo.
product-revenue-account-unknown = Il conto di ricavo di questo articolo non è stato trovato o non appartiene a questa società.
product-revenue-account-inactive = Il conto di ricavo di questo articolo è archiviato.
product-revenue-account-not-revenue = Il conto di questo articolo non è un conto di ricavo.

# Story 16-2b (#144) — selettore del conto di ricavo sulla scheda articolo.
product-form-revenue-account = Conto di ricavo
product-form-revenue-account-help = Facoltativo. Se lasciato vuoto, le righe di fattura create da questo articolo seguono il conto di ricavo predefinito della società.
product-form-revenue-account-load-error = Impossibile caricare il piano dei conti. Il conto di ricavo dovrà essere inserito tramite il suo identificatore, oppure ricaricare la pagina.


# Story 16-3a (#151) — saisie des coordonnées dans les réglages société.
settings-field-company-phone = Telefono
settings-company-phone-help = Numero di telefono visualizzato sulle vostre fatture. Vuoto = riga omessa.
settings-field-company-website = Sito web
settings-company-website-help = Indirizzo del vostro sito, visualizzato sulle fatture. Vuoto = riga omessa.
settings-company-contact-saved = Recapiti salvati
settings-company-contact-conflict = Conflitto di versione — i dati sono stati ricaricati, riprovare.
settings-company-contact-conflict-reload-failed = Conflitto di versione e ricaricamento impossibile — ricaricare la pagina.
error-company-phone-too-long = Il numero di telefono della società è troppo lungo (al massimo 50 caratteri).
error-company-website-too-long = L'indirizzo del sito web della società è troppo lungo (al massimo 255 caratteri).
error-invoice-pdf-header-overflow = L'intestazione del documento non entra nella pagina. Eliminate un recapito — telefono, e-mail o sito web — nelle impostazioni: accorciarlo non libera spazio, ogni recapito occupa una riga intera. Oppure riducete il numero di righe dell'indirizzo del destinatario.

# Story 23-2 (#283) — parità it-CH : le 57 chiavi che esistevano solo in fr-CH.
# Termini RILEVATI nel catalogo, non inventati : scrittura contabile
# (error-journal-entry-linked-to-invoice), esercizio (error-no-fiscal-year), Convalidata
# (invoice-status-validated), Ricavo (account-type-revenue),
# ⚠️ Attiva/Inattiva (reconciliation-rules-labels-active, api-keys-labels-status-active) — le STATUT,
# et NON Attivo/Inattivo qui est la classe di bilancio (account-type-asset). L'italien fait la même
# distinction que l'anglais entre Active et Asset, et le sujet « aliquota » est féminin.
# Descrizione vs Etichetta (journal-entries-col-description / reconciliation-rules-labels-label).
# ⚠️ « Date de bascule » NON è l'interruttore del glossario : è il cambiamento di aliquota.
error-configuration-required = Configurazione incompleta: configura le impostazioni di fatturazione prima di convalidare.
error-fiscal-year-invalid = Nessun esercizio aperto copre questa data.
invoice-error-already-validated = Questa fattura è già convalidata.
invoice-error-configuration-required = Configura i conti predefiniti in Impostazioni > Fatturazione prima di convalidare una fattura.
invoice-error-configuration-required-non-admin = Chiedi al tuo amministratore di configurare i conti predefiniti di fatturazione.
invoice-error-fiscal-year-invalid = Nessun esercizio aperto copre la data della fattura.
invoice-journal-entry-description = Fattura { $invoiceNumber } - { $contactName }
invoice-number-label = Numero
invoice-status-validated-label = Convalidata
invoice-validate-button = Convalida
invoice-validate-confirm-body = Una volta convalidata, questa fattura sarà immutabile, riceverà un numero definitivo e genererà una scrittura contabile. Continuare?
invoice-validate-confirm-title = Convalida la fattura
invoice-validate-success = Fattura convalidata — { $invoiceNumber }
invoice-validate-success-body = La fattura { $invoiceNumber } è ora convalidata e immutabile. La scrittura contabile associata è stata generata.
invoice-view-journal-entry-link = Visualizza la scrittura contabile
settings-invoicing-description-template = Descrizione della scrittura contabile
settings-invoicing-format-help = Segnaposto: {"{"}YEAR{"}"}, {"{"}FY{"}"}, {"{"}SEQ{"}"}, {"{"}SEQ:NN{"}"}
settings-invoicing-format-invalid = Formato non valido
settings-invoicing-format-label = Formato di numerazione
settings-invoicing-format-preview = Anteprima
settings-invoicing-numbering-title = Numerazione
settings-invoicing-seq-range = (NN tra 1 e 10)
settings-invoicing-description-help = Segnaposto: {"{"}YEAR{"}"}, {"{"}INVOICE_NUMBER{"}"}, {"{"}CONTACT_NAME{"}"}.
settings-invoicing-description-invalid = Testo di registrazione non valido
settings-invoicing-default-accounts-title = Conti predefiniti
settings-invoicing-select-none = — Selezionare —
settings-invoicing-load-error = Errore di caricamento
settings-invoicing-save-error = Errore durante il salvataggio
settings-invoicing-journal = Giornale
settings-invoicing-receivable-account = Conto crediti verso clienti (Attivo)
settings-invoicing-revenue-account = Conto ricavi (Ricavo)
settings-invoicing-save = Salva
settings-invoicing-save-success = Configurazione salvata
settings-invoicing-title = Impostazioni — Fatturazione
settings-vat-rates-link = Configura le aliquote IVA e le loro date di validità (i cambiamenti di aliquota sono gestiti nel tempo).
vat-rates-active = Attiva
vat-rates-change = Cambia l'aliquota
vat-rates-change-error = Il cambiamento di aliquota non è riuscito.
vat-rates-change-hint = La vecchia aliquota terminerà alla data di cambiamento, e la nuova aliquota avrà effetto da tale data.
vat-rates-changed = Aliquota aggiornata.
vat-rates-col-actions = Azioni
vat-rates-col-from = Valida dal
vat-rates-col-rate = Aliquota
vat-rates-col-status = Stato
vat-rates-col-to = Fino al
vat-rates-create-error = La creazione non è riuscita.
vat-rates-created = Aliquota IVA creata.
vat-rates-deactivate = Disattiva
vat-rates-deactivate-confirm = Disattivare questa aliquota? Non sarà più proposta durante l'inserimento ma resterà nello storico.
vat-rates-deactivate-error = La disattivazione non è riuscita.
vat-rates-deactivated = Aliquota disattivata.
vat-rates-empty = Nessuna aliquota configurata.
vat-rates-field-category = Categoria
vat-rates-field-from = Valida dal
vat-rates-field-label = Etichetta (facoltativa)
vat-rates-field-new-rate = Nuova aliquota (%)
vat-rates-field-rate = Aliquota (%)
vat-rates-field-switch-date = Data di cambiamento
vat-rates-field-to = Fino al (facoltativo)
vat-rates-inactive = Inattiva
vat-rates-load-error = Impossibile caricare le aliquote IVA.
vat-rates-new = Nuova aliquota
vat-rates-subtitle = Configura le aliquote IVA e le loro date di validità. Le vecchie aliquote restano applicate alle operazioni anteriori.
vat-rates-title = Aliquote IVA

# Story 23-3 (#316) — fatture fornitori : 119 chiavi.
# Termini RILEVATI : fattura fornitori, Scadenza (invoice-pdf-due-date), Annullata
# (invoice-status-cancelled), Costo (account-type-expense), IVA incl./esclusa
# (invoice-pdf-total-ttc / vat-purchase-ht), esercizio.
# Termini TRANCIATI dalla parte B del glossario : fattura QR (terminologia ufficiale SIX),
# documento giustificativo, scartare/scartata, completare, immagine.
# ⚠️ `imported-supplier-invoices-save` = `Crea una fattura`. Il francese diceva « Valider la
# facture », errato due volte : il codice CREA la fattura (`complete_import`, passo 7), e
# `invoice-validate-confirm-title` portava GIÀ quel testo per la convalida contabile. Deciso
# da Guy il 2026-08-20 : « Créer une facture », l'uso di Bexio. ⚠️ Verbo RILEVATO su
# `homepage-invoices-action` (`Creare una fattura`), ma reso all'IMPERATIVO come tutti i
# pulsanti di questo dominio (`Registra`, `Completa`, `Scarta`, `Paga`).
# ⚠️ Accordi al FEMMINILE con « fattura » : creata, scartata, importata, annullata.
imported-supplier-invoices-add-line = Aggiungi una riga
imported-supplier-invoices-amount = Importo
imported-supplier-invoices-complete = Completa
imported-supplier-invoices-completed = Fattura creata.
imported-supplier-invoices-completed-hint = Fattura #{$id} registrata.
imported-supplier-invoices-completed-reload-failed = Importazione eseguita, ma non è stato possibile ricaricare l'elenco — aggiorna la pagina.
imported-supplier-invoices-discard = Scarta
imported-supplier-invoices-discard-confirm = Scartare questa fattura importata? Il documento giustificativo resta conservato.
imported-supplier-invoices-discard-conflict = Questa fattura è già stata completata o scartata da un'altra sessione.
imported-supplier-invoices-discard-failed = Impossibile scartare la fattura.
imported-supplier-invoices-discarded = Fattura scartata.
imported-supplier-invoices-doc-failed = Download impossibile.
imported-supplier-invoices-doc-gone = Il documento giustificativo non è stato ripristinato.
imported-supplier-invoices-empty = Nessuna fattura da completare.
imported-supplier-invoices-err-amount = Il totale delle righe ({$actual}) non corrisponde all'importo del QR ({$expected}).
imported-supplier-invoices-err-currency = Valuta non supportata (solo CHF).
imported-supplier-invoices-err-fiscal-year = Nessun esercizio aperto copre questa data.
imported-supplier-invoices-err-form = Verifica il fornitore, la data e ogni riga (descrizione, importi, conto di costo).
imported-supplier-invoices-err-generic = Errore imprevisto.
imported-supplier-invoices-err-iban-ref = Incoerenza tra l'IBAN e il riferimento QRR.
imported-supplier-invoices-err-not-found = Fattura importata non trovata.
imported-supplier-invoices-err-not-pending = Questa fattura è già stata completata o scartata.
imported-supplier-invoices-error-unknown = Importazione non riuscita ({$code})
imported-supplier-invoices-field-date = Data della fattura
imported-supplier-invoices-field-due = Scadenza
imported-supplier-invoices-field-number = N. fattura fornitore
imported-supplier-invoices-field-supplier = Fornitore
imported-supplier-invoices-import = Importa la cartella
imported-supplier-invoices-import-failed = Errore imprevisto durante l'importazione.
imported-supplier-invoices-import-running = Un'importazione è già in corso. Riprova tra qualche istante.
imported-supplier-invoices-importing = Importazione in corso…
imported-supplier-invoices-line-account = Conto
imported-supplier-invoices-line-desc = Descrizione
imported-supplier-invoices-line-qty = Qtà
imported-supplier-invoices-line-ht = Prezzo unitario IVA esclusa
imported-supplier-invoices-lines = Righe
imported-supplier-invoices-mismatch = differenza da correggere
imported-supplier-invoices-no-source-doc = Per questa fattura non è stato importato alcun documento giustificativo.
imported-supplier-invoices-reference = Rif.
imported-supplier-invoices-reload-failed = Non è stato possibile ricaricare l'elenco — aggiorna la pagina.
imported-supplier-invoices-report-accepted = {$n} fattura/e importata/e.
imported-supplier-invoices-report-failed = {$n} errore/i:
imported-supplier-invoices-save = Crea una fattura
imported-supplier-invoices-source-doc-failed = Download impossibile.
imported-supplier-invoices-source-doc-gone = Il documento giustificativo non è stato ripristinato.
imported-supplier-invoices-subcentime = L'importo del QR ({$amount}) contiene frazioni di centesimo — irraggiungibile con righe al centesimo esatto. Raccomandazione: scartare questa fattura.
imported-supplier-invoices-sum-ttc = Totale delle righe IVA inclusa
imported-supplier-invoices-target = obiettivo QR
imported-supplier-invoices-title = Importa fatture
imported-supplier-invoices-to-complete = Fatture da completare
imported-supplier-invoices-view-doc = Visualizza il documento giustificativo
imported-supplier-invoices-view-source = Visualizza la fattura originale
imported-supplier-invoices-error-unsupported-file-type = Tipo di file non supportato
imported-supplier-invoices-error-file-too-large = File troppo grande
imported-supplier-invoices-error-symlink-rejected = Collegamento simbolico rifiutato
imported-supplier-invoices-error-duplicate = Già importato (duplicato)
imported-supplier-invoices-error-no-qr-code-found = Nessuna fattura QR rilevata
imported-supplier-invoices-error-invalid-spc-payload = QR illeggibile (formato non SPC)
imported-supplier-invoices-error-invalid-iban = IBAN del creditore non valido
imported-supplier-invoices-error-pdf-render-error = PDF illeggibile
imported-supplier-invoices-error-file-read-error = Impossibile leggere il file
imported-supplier-invoices-error-field-too-long = Un campo della fattura QR supera la lunghezza consentita
supplier-invoices-add-line = Aggiungi una riga
supplier-invoices-cancel = Annulla la fattura
supplier-invoices-cancel-confirm = Annullare questa fattura fornitore?
supplier-invoices-col-date = Data
supplier-invoices-col-due = Scadenza
supplier-invoices-col-number = N.
supplier-invoices-col-status = Stato
supplier-invoices-col-qty = Qtà
supplier-invoices-col-vat = IVA
supplier-invoices-col-supplier = Fornitore
supplier-invoices-col-total = IVA incl.
supplier-invoices-detail-project = Progetto
supplier-invoices-detail-reference = Riferimento
supplier-invoices-detail-title = Fattura fornitore
supplier-invoices-empty = Nessuna fattura fornitore registrata.
supplier-invoices-err-lines = Ogni riga richiede una descrizione, un importo e un conto di costo.
supplier-invoices-err-supplier = Seleziona un fornitore.
supplier-invoices-field-date = Data della fattura
supplier-invoices-field-due = Scadenza
supplier-invoices-field-expected-amount = Importo previsto IVA inclusa (facoltativo)
supplier-invoices-field-iban = IBAN / QR-IBAN (facoltativo)
supplier-invoices-field-number = N. fattura fornitore
supplier-invoices-field-project = Progetto (facoltativo)
supplier-invoices-field-qr-iban = QR-IBAN (facoltativo)
supplier-invoices-field-reference = Riferimento (facoltativo)
supplier-invoices-field-supplier = Fornitore
supplier-invoices-form-close = Chiudi
supplier-invoices-line-account = Conto
supplier-invoices-line-desc = Descrizione
supplier-invoices-line-ht = IVA escl.
supplier-invoices-line-total = Totale IVA esclusa
supplier-invoices-lines = Righe
supplier-invoices-new = Registra una fattura
supplier-invoices-paid-info = Fattura pagata.
supplier-invoices-pay-account-ph = Conto di contropartita
supplier-invoices-pay-bank-ph = Conto bancario di origine
supplier-invoices-pay-date = Data di pagamento
supplier-invoices-pay-err-account = Seleziona un conto.
supplier-invoices-pay-err-bank = Seleziona un conto bancario.
supplier-invoices-pay-internal = Conto interno (cassa, carta…)
supplier-invoices-pay-submit = Paga
supplier-invoices-pay-title = Paga la fattura
supplier-invoices-pay-transfer = Bonifico bancario
supplier-invoices-project-none = — Nessuno
supplier-invoices-save = Salva
supplier-invoices-scan = Scansiona una fattura QR
supplier-invoices-scan-detected = Creditore rilevato
supplier-invoices-scan-failed = Impossibile leggere questa immagine.
supplier-invoices-scan-hint = Carica un'immagine della fattura QR per precompilare IBAN, riferimento e importo.
supplier-invoices-scan-no-qr = Nessun codice QR rilevato su questa immagine.
supplier-invoices-scan-ok = Fattura QR letta — dati precompilati.
supplier-invoices-scan-running = Lettura…
supplier-invoices-scan-too-large = Immagine troppo grande (max 15 MB).
supplier-invoices-status-open = Aperta
supplier-invoices-status-paid = Pagata
supplier-invoices-status-cancelled = Annullata

# Story 23-3b (#316) — etichette hardcoded, cf. fr-CH per la derivazione di ogni termine.
credit-notes-status-cancelled = Annullata
credit-notes-status-draft = Bozza
credit-notes-status-issued = Emessa
credit-notes-col-date = Data
credit-notes-col-description = Descrizione
credit-notes-col-line-total = Totale IVA esclusa
credit-notes-col-number = N. nota di credito
credit-notes-col-qty = Qtà
credit-notes-col-status = Stato
credit-notes-col-total = Totale IVA esclusa
credit-notes-col-unit-price = Prezzo unitario
credit-notes-col-vat = IVA %
credit-notes-confirm-body = Verrà creata e contabilizzata immediatamente una nota di credito totale: storna la scrittura di questa fattura (il saldo del cliente torna a zero) e la fattura passa allo stato «annullata». L’azione è definitiva. Continuare?
credit-notes-create-button = Crea una nota di credito
credit-notes-create-error = Creazione della nota di credito non riuscita
credit-notes-created = Nota di credito creata
credit-notes-download-pdf = Stampa / Scarica PDF
credit-notes-empty = Nessuna nota di credito. Creane una da una fattura convalidata per annullarla.
credit-notes-title = Note di credito
credit-notes-detail-title = Nota di credito
credit-notes-view-entry = Visualizza la scrittura contabile
credit-notes-view-invoice = Visualizza la fattura annullata
credit-notes-view-list = Visualizza le note di credito
nav-bank-import = Importa estratti conto
nav-invoicing = Fatturazione
nav-journal-entries = Scritture
nav-reconciliation = Riconciliazione
nav-reports = Rapporti
nav-users = Utenti
payment-batches-failed-already-in-generated-batch = Già in un lotto creato
payment-batches-failed-invalid-iban = IBAN non valido
payment-batches-failed-invalid-qr-iban = QR-IBAN non valido
payment-batches-failed-no-payment-coordinates = Nessuna coordinata di pagamento (IBAN/QR-IBAN)
payment-batches-failed-supplier-invoice-not-found = Fattura non trovata
payment-batches-failed-supplier-invoice-not-open = Fattura non aperta
payment-batches-status-cancelled = Annullato
payment-batches-status-confirmed = Confermato
payment-batches-status-generated = Creato

# Story 23-4 (#316) — rollout `settings` + `payment-batches` + `onboarding`, cf. fr-CH.
closing = Chiusura…
email-templates-badge-custom = Personalizzato
email-templates-badge-default = Predefinito
email-templates-body-label = Corpo del messaggio
email-templates-conflict = Conflitto di versione — il modello è stato ricaricato
email-templates-load-error = Errore di caricamento
email-templates-save-error = Errore durante il salvataggio
email-templates-restore-error = Errore durante il ripristino
email-templates-lang-tablist = Lingua
email-templates-restore = Ripristina il predefinito
email-templates-restore-confirm-action = Ripristina il predefinito
email-templates-restore-confirm-body = Il tuo testo personalizzato per questa combinazione sarà eliminato e sostituito dal modello predefinito. Questa azione è irreversibile.
email-templates-restore-confirm-title = Ripristinare il modello predefinito?
email-templates-restored = Modello predefinito ripristinato
email-templates-restoring = Ripristino…
email-templates-saved = Modello salvato
email-templates-subject-label = Oggetto
email-templates-subtitle = Personalizza il contenuto delle e-mail inviate ai tuoi clienti, per lingua. Se non modifichi nulla, viene utilizzato automaticamente un modello predefinito.
email-templates-title = Modelli di e-mail
email-templates-unknown-variables = Il modello contiene variabili sconosciute
email-templates-unknown-variables-list = Variabili sconosciute:
email-templates-variables-hint = Inserisci queste variabili nell'oggetto o nel corpo; saranno sostituite all'invio.
email-templates-variables-title = Variabili disponibili
nav-credit-notes = Note di credito
nav-email-templates = Modelli di e-mail
nav-projects = Progetti
nav-supplier-invoices-import = Importa fatture
onboarding-address-npa-city-required = NPA e località sono obbligatori
onboarding-already-finalized = La configurazione iniziale è già stata completata. Ricarica la pagina.
onboarding-field-name-hint = azienda — altrimenti compila nome + cognome
onboarding-finalize-incomplete = Il completamento non è stato confermato. Ricarica la pagina.
onboarding-mode-error = Errore nella scelta della modalità
onboarding-demo-load-error = Errore durante il caricamento dei dati dimostrativi
onboarding-setup-start-error = Errore all'avvio della configurazione
onboarding-org-type-error = Errore nella scelta del tipo
onboarding-accounting-lang-error = Errore nella scelta della lingua contabile
onboarding-name-required = Nome (o nome + cognome) obbligatorio
onboarding-address-save-error = Errore durante il salvataggio dei dati
onboarding-finalize-error = Finalizzazione non riuscita — riprovare.
onboarding-bank-required = Nome della banca e IBAN sono obbligatori
onboarding-bank-save-error = Errore durante il salvataggio del conto bancario
onboarding-bank-skip-error = Errore durante l'omissione del conto bancario
onboarding-save = Salva
payment-batches-cancel = Annulla il lotto
payment-batches-cancel-confirm = Annullare questo lotto? Le fatture tornano pagabili.
payment-batches-col-date = Esecuzione
payment-batches-col-id = Lotto
payment-batches-col-status = Stato
payment-batches-col-total = Totale
payment-batches-confirm = Conferma il lotto
payment-batches-confirm-hint = Una volta eseguito il bonifico nell'e-banking, conferma per contabilizzare i pagamenti.
payment-batches-confirm-title = Conferma il pagamento
payment-batches-confirmed-at = Confermato il
payment-batches-created = Lotto creato
payment-batches-detail-date = Data di esecuzione
payment-batches-detail-title = Lotto di pagamento
payment-batches-download = Scarica il file pain.001
payment-batches-empty = Nessun lotto di pagamento.
payment-batches-err-bank = Seleziona un conto bancario di origine.
payment-batches-err-invoices = Seleziona almeno una fattura.
payment-batches-field-bank = Conto bancario di origine
payment-batches-field-date = Data di esecuzione desiderata
payment-batches-form-close = Chiudi
payment-batches-generate = Crea il lotto
payment-batches-item-invoice = Fattura
payment-batches-line-amount = Importo
payment-batches-lot = Lotto
payment-batches-msg-id = MsgId
payment-batches-new = Crea un lotto di bonifici
payment-batches-no-open = Nessuna fattura fornitore aperta.
payment-batches-open = apri e scarica il file
payment-batches-payment-date = Data di pagamento effettiva
payment-batches-rejected = Fatture non incluse:
payment-batches-select-invoices = Fatture da pagare tramite bonifico
projects-archive = Archivia
projects-archive-error = Operazione impossibile.
projects-archived = Progetto archiviato.
projects-archived-tag = archiviato
projects-created = Progetto creato.
projects-edit = Modifica
projects-empty = Nessun progetto. Crea il tuo primo progetto per iniziare.
projects-err-code = Il codice del progetto è obbligatorio.
projects-err-name = Il nome del progetto è obbligatorio.
projects-field-code = Codice
projects-field-description = Descrizione (facoltativo)
projects-field-end = Fine
projects-field-name = Nome
projects-field-parent = Progetto superiore (facoltativo)
projects-field-start = Inizio
projects-form-create = Nuovo progetto
projects-form-edit = Modifica il progetto
projects-load-error = Impossibile caricare i progetti.
projects-new = Nuovo progetto
projects-parent-none = — Nessuno (progetto radice)
projects-save = Salva
projects-save-error = Salvataggio impossibile.
projects-show-archived = Mostra i progetti archiviati
projects-subtitle = Raggruppa le tue spese e i tuoi ricavi per progetto (ristrutturazione, investimento) per analizzarli separatamente.
projects-title = Progetti
projects-unarchive = Ripristina dall'archivio
projects-unarchived = Progetto ripristinato dall'archivio.
projects-updated = Progetto aggiornato.
settings-email-templates-link = Personalizza il contenuto delle e-mail inviate ai tuoi clienti (oggetto e corpo, per lingua).
# --- Libro mastro (Story 24-1) ---
reports-ledger-title = Libro mastro
reports-ledger-tab = Libro mastro
reports-ledger-description = L'estratto di un conto: ciò che ne compone il saldo, riga per riga.
reports-ledger-account-label = Conto
reports-ledger-all-accounts = Tutti i conti movimentati
reports-ledger-from = Dal
reports-ledger-to = Al
reports-ledger-opening = Saldo di apertura
reports-ledger-closing = Saldo di chiusura
reports-ledger-movements-total = Totale dei movimenti
reports-ledger-column-piece = Documento
reports-ledger-column-journal = Giornale
reports-ledger-column-counterpart = Contropartita
reports-ledger-column-running = Saldo progressivo
reports-ledger-archived = archiviato
reports-ledger-unnatural = Saldo contrario alla natura del conto
reports-ledger-unnatural-hint = Questo conto presenta un saldo dal lato opposto alla sua natura. Da verificare.
reports-ledger-fy-break = Chiusura dell'esercizio — il saldo riparte da zero
reports-ledger-no-movement = Nessun movimento nel periodo. Il saldo di apertura resta dovuto.
reports-ledger-empty = Nessun conto da visualizzare per questo periodo.
reports-ledger-open-from-balance = Vedi il dettaglio nel libro mastro
reports-ledger-truncated = Sono visualizzate solo le prime { $shown } righe su { $total }. L'esportazione le contiene tutte.
reports-ledger-include-zero = Includere i conti senza movimenti
reports-ledger-instruction = Scegliete un periodo, poi cliccate su Genera. Il conto è facoltativo.

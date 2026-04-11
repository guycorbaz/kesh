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

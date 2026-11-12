# Epic 35 — registre des choix autonomes du kickoff et des fiches 35-2, 35-3 (clé `e350`)

Registre propre au kickoff (story 35-0), **versé par union** par l'orchestrateur. Chaque choix : contexte, option
retenue, options écartées, réversibilité. Lignes citées sur `release/0.17.6` (`4e1c768a`).

---

### C-e350-1 — #944 n'entre pas dans la 35-2

- **Contexte** : le mandat laisse #944 « en option à trancher » pour la story du refus du PDF. Lu à l'issue, #944 n'est
  pas un point mais huit : ligne verticale, ciseaux ou « Separate before paying in » (`crates/kesh-qrbill/src/pdf.rs:728-744`,
  « Scissors glyph … omitted in v0.1 »), place de *Payable by*, largeur du message, très longues adresses, coupe de ligne
  calée sur les capitales (27 `W` débordent sur le QR), *Acceptance point* recouvert, cadre vide du débiteur absent,
  type K. Impact I3 : le code QR se lit, la banque accepte.
- **Retenu** : #944 hors de la 35-2, laissé ouvert, proposé à l'Epic 36 comme story à lui seul.
- **Écartées** : (a) #944 entier dans la 35-2 : autre crate (`kesh-qrbill`), golden tests et captures du PDF, boucle
  complète ; la story qui doit sortir avant le 2026-11-30 deviendrait la plus lourde de l'epic ; (b) la seule ligne
  verticale avec le texte « Separate before paying in » : découpe l'issue en laissant ses autres points, et touche
  quand même le rendu de chaque QR-facture à une semaine de la première facture réelle.
- **Réversible** : oui ; une story 35-9 peut le prendre si l'epic finit tôt.

### C-e350-2 — #537 : le type se revérifie à l'usage, pas au retypage

- **Contexte** : #537 propose deux remèdes, refuser ou signaler le retypage d'un compte désigné, ou revérifier le type à
  l'usage. Lu au code : `lock_designated_accounts_in_tx` lit `id, number, active, postable` sans `account_type`
  (`company_invoice_settings.rs:960-970`), et son doc-comment écrit l'angle mort (`:902-904`). Le retypage passe par
  `accounts::update` (`repositories/accounts.rs:622`), dont `check_role_account_type` ne contrôle que la paire
  rôle/type **demandée** : retirer le rôle et changer le type dans la même requête passe.
- **Retenu** : revérifier le type dans `DesignatedAccountsSnapshot::check_written`, pour les quatre rôles désignés
  (créance et TVA due à la validation d'une facture ; créanciers et TVA récupérable à celle d'une facture fournisseur),
  avec un refus nommé, typé et traduit. Le commentaire de #537 (« même forme côté dette ») est ainsi traité avec elle.
- **Écartées** : (a) refuser le retypage d'un compte désigné : une seconde garde, sur un autre chemin, qui laisse
  passer tout autre moyen de changer le type ; (b) avertir au retypage : utile, mais ce n'est pas la garde ; à proposer
  en issue si la story le juge utile ; (c) l'avoir : hors de la story, comme le dit le doc-comment (C35 de la 15-5d) —
  une facture émise doit rester annulable.
- **Réversible** : oui ; un refus en plus, aucune donnée écrite.

### C-e350-3 — #955 : le serveur nomme le champ, l'écran reprend son message

- **Contexte** : `INVOICE_NOT_PDF_READY` est rendu sans `details` (`errors.rs:2073-2075`, `build_response` à `:1420`) ;
  le message vient de `map_qrbill_error` (`invoice_pdf_service.rs:494-541`), en français en dur et avec des noms
  techniques (`debtor.name`). L'écran de la fiche le remplace par une clé fixe (`pdf-error.ts:18`, `:44-45`) ; le lot de
  rappels n'affiche que le code (`ReminderBatchReport.svelte:30`) ; l'envoi par e-mail affiche le message brut
  (`invoices/[id]/+page.svelte:866`).
- **Retenu** : le refus porte des `details` structurés (champ, raison, caractère et son point de code) et un message
  **résolu au serveur** dans la langue de l'installation, qui nomme le champ en mots de l'utilisateur (« la localité du
  client »). **Tous** les messages de ce code passent par une clé (inventaire clos des sites, fiche 35-2), si bien que
  l'écran peut reprendre le message du serveur pour `INVOICE_NOT_PDF_READY`, comme il le fait déjà pour les codes de
  `SERVER_RESOLVED_CODES` (`pdf-error.ts:36`). Le lot de rappels montre la même phrase. Forme exacte : la fiche 35-2.
  *(Précisé à la rédaction de la fiche 35-2 : la première rédaction réservait le message du serveur au cas où les
  `details` sont là ; or les refus sans champ — adresse incomplète, compte bancaire principal absent — sont eux aussi
  remplacés aujourd'hui par la phrase fixe, et l'un d'eux, `"qrbill-error-qrr-generation"`
  (`invoice_pdf_service.rs:338`), envoie une clé nue en guise de message.)*
- *(Annoté le 2026-11-12, validation P4 de la 35-2a : **suspendu par C-e350-11** pour les `details` ; le message résolu
  au serveur et repris par l'écran reste.)*
- **Écartées** : (a) afficher le message brut du serveur tel qu'il est : technique et non traduit ; (b) résoudre au
  frontend à partir des `details` seuls : deux tables de libellés de champ à tenir (serveur et écran) pour un même
  message.
- **Réversible** : oui ; `details` s'ajoute au contrat sans rien retirer.

### C-e350-4 — 35-2 et 35-3 en parallèle malgré trois fichiers communs

- **Contexte** : les deux doivent sortir avant le 2026-11-30 ; elles touchent `crates/kesh-api/src/errors.rs`, les
  quatre `messages.ftl` et `user-manual.tex`, à des passages distincts.
- **Retenu** : parallèle, union à l'intégration, PDF du manuel régénéré une fois après l'union.
- **Écartée** : en série (35-3 puis 35-2) : sûr, mais retarde la seconde d'une boucle complète.
- **Réversible** : oui.

### C-e350-5 — La règle d'arrêt vaut pour toute migration

- **Contexte** : le mandat dit « migration qui empêche le retour à 0.17.6 → s'arrêter ». Le `CLAUDE.md` (§ *Migration
  breaking policy*, P1, constaté à la release 0.13.3) établit que toute migration empêche le binaire précédent de
  démarrer.
- **Retenu** : toute migration déclenche l'arrêt de l'orchestrateur ; une fiche qui en envisage une le dit avant de
  l'écrire.
- **Écartée** : ne s'arrêter que sur une migration *breaking* (`kesh_version_min_required` relevé) : contredit le fait
  constaté, le retour à la 0.17.6 passerait alors par la sauvegarde.
- **Réversible** : c'est à Guy de l'assouplir.

### C-e350-6 — Ordre après la tête

- **Retenu** : 35-4, 35-6, 35-5 (I1), puis 35-7, 35-8, 35-1 (I2), 35-1 en dernier (format par défaut gardé, probabilité
  basse), 35-8 après 35-5 et 35-3 (fichiers communs).
- **Réversible** : oui.

### C-e350-7 — Le solde du reste reste hors de la 35-3, avec une issue

- **Contexte** : validation P1 de la 35-3, V1-1. Le solde du reste d'un règlement relit la TVA due par
  `vat_payable_account_for_write` (`company_invoice_settings.rs:661-691`, « sans contrainte de type », `:663-664`),
  appelé à `invoice_settlements_write.rs:679-680` ; un compte de TVA due retypé y reçoit la correction de TVA.
- **Retenu** : hors de la 35-3 ; issue à ouvrir par l'orchestrateur. *(Annoté le 2026-11-12, validation P2 de la 35-3 :
  ouverte, **#1044**.)*
- **Raisons** : ce n'est pas la même garde — une relecture `FOR UPDATE` dont le refus est `ConfigurationRequired`, et
  non l'instantané de `check_written` ; poser le refus nommé y change le contrat d'une autre route (règlement) et son
  écran, dans un fichier que la 0.17.6 vient de remanier (34-5) ; le chemin n'existe que pour une facture **à TVA**, et
  Guy n'est pas assujetti (famille des P1 de la TVA reportés, comme #525).
- **Écartée** : l'inclure, ce qui allonge une story qui doit sortir avant le 2026-11-30.
- **Réversible** : oui ; la constante publique de la 35-3 (D4) servira à cette issue.

### C-e350-8 — Règle d'union des fichiers communs à la 35-2 et à la 35-3

- **Contexte** : validations P1 des deux fiches (35-2 V1-13, 35-3 V1-7). Les deux stories, en parallèle, écrivent aux
  mêmes fichiers ; « clés distinctes » ne protège pas d'un conflit si les deux ajoutent au même endroit.
- **Retenu**, recopié dans les deux fiches et au fichier d'epic :
  1. **`CHANGELOG.md`** : chaque branche crée, si elle manque, la section `## [0.17.7] — Non publié` juste au-dessus de la
     première section `## [` existante, avec la seule rubrique `### Corrigé` ; elle y écrit **un** point par issue. À
     l'intégration, une seule section et une seule rubrique sont gardées, les points dans l'ordre des stories (35-2 puis
     35-3), l'avertissement de la 35-3 en tête de son point. *(Annoté le 2026-11-12, validation P4 de la 35-2a : la 35-2b
     complète le point #955 de la 35-2a, sans en écrire un second.)*
  2. **`messages.ftl` ×4** : jamais en fin de fichier. 35-2 : à côté du bloc `invoice-pdf-error-*` (fr-CH `:967-983` sur
     `release/0.17.6`) ; 35-3 : juste après `error-designated-account-not-postable`.
  3. **G9** (`crates/kesh-api/tests/textes_coherents.rs`, `TABLE: [(&str, Cles); 8]`) : la 35-3 ajoute ses clés **en fin
     du bloc `crates/kesh-api/src/errors.rs`** existant, sans changer la longueur ; la 35-2 ajoute une **entrée neuve en fin
     de table** pour `crates/kesh-api/src/routes/invoice_pdf_service.rs` (longueur 9), et ses éventuelles clés
     d'`errors.rs` en fin du même bloc, **après un commentaire `// Story 35-2`** (la 35-3 met `// Story 35-3`). À l'union,
     la longueur est recomptée.
     *(Annoté le 2026-11-12, validation P3 de la 35-2a : la 35-2a ajoute **deux** entrées, `invoice_pdf_service.rs` et
     `credit_notes.rs`, et aucune clé à `errors.rs` : la table passe de 8 à **10**. Les ajouts de la 35-3 au bloc
     `errors.rs` ne se heurtent plus à ceux de la 35-2.)*
  4. **`crates/kesh-api/src/errors.rs`** : 35-2, le bras `InvoiceNotPdfReady` et ce qu'il appelle ; 35-3, un bras `DbError`
     neuf juste après `DesignatedAccountsNotPostable` (`:4571`). Aucun reformatage hors des passages touchés.
     *(Annoté le 2026-11-12, validation P4 de la 35-2a : pour la 35-2, **suspendu par C-e350-11** — la 35-2a ne touche plus
     `errors.rs`.)*
  5. **`docs/api-external.md`** : 35-3, une ligne juste après `ACCOUNT_NOT_POSTABLE` ; 35-2, sa ligne juste après
     `SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE` (`:1058` sur `release/0.17.6`), en fin du premier groupe des `400`.
  6. **`user-manual.tex`** : paragraphes distincts ; chaque branche régénère son PDF pour ses contrôles, mais à l'union le
     PDF n'est **jamais fusionné** : il est régénéré depuis le `.tex` uni, puis contrôlé aplati.
- **Réversible** : oui.

### C-e350-9 — La 35-2 se découpe en 35-2a et 35-2b

- **Contexte** : validation P2 de la 35-2, P1 5 MEDIUM → P2 5 MEDIUM, dont quatre nés de la remédiation P1 (recyclage) ;
  le contrat `details` grossit (onze raisons) sur plus de cinq modules. Le `CLAUDE.md` (§ *Règle de splitting
  préventif*, amendement D5) fait découper le recyclage.
- **Retenu** : **35-2a** (routes unitaires, contrat `details`, adresse, pays, écran, manuel, E2E ; échéance du
  2026-11-30) et **35-2b** (lot de rappels, partant de la 35-2a fusionnée). La fiche 35-2 passe en `split`.
- **Écartées** : (a) élargir la 35-2 : la règle l'interdit sur un signal de recyclage ; (b) découper par couche
  (serveur, écran) : le contrat se prouverait sans son écran.
- **Réversible** : oui. Signal à déclarer à Guy par l'orchestrateur.

### C-e350-10 — Contrat de `details` des refus `INVOICE_NOT_PDF_READY`

- **Contexte** : validation P2, V2-2 et V2-4 : la forme n'était fixée que pour les refus de champ ; un agent qui écrit par
  une clé REST ne distinguait pas « compte principal absent » d'« adresse du client incomplète » sans lire le message.
- **Retenu** : `details` **toujours** présent, `reason` toujours présent, en SCREAMING_SNAKE : `INVALID_CHARSET`,
  `EMPTY`, `TOO_LONG`, `INVALID_COUNTRY` (avec `field` et leurs membres), `CLIENT_ADDRESS_MISSING`, `CONTACT_MISSING`,
  `NO_PRIMARY_BANK`, `INVALID_BANK_DATA`, `INVALID_AMOUNT`, `INVALID_CURRENCY`, `INTERNAL_PAYLOAD`. Le lot (35-2b) ajoute
  `message` au même objet. Une adresse incomplète nomme son champ (`EMPTY`, `debtor.town`…) ; le pays nomme sa partie.
  Tableau complet : D2 de la 35-2a.
- **Écartée** : `details` pour les seuls refus de champ, les autres restant à lire dans le message — écrit comme limite,
  ce serait un contrat que l'API dit elle-même de ne pas lire (`api-external.md`, « Fiez-vous au champ `code` »).
- **Réversible** : ajouter une raison l'est ; en retirer une ne l'est plus une fois publiée.
- *(Annoté le 2026-11-12, validation P3 de la 35-2a : **suspendu** par C-e350-11 ; la proposition part dans **#1045**.)*

### C-e350-11 — La 35-2a revient au message, sans contrat `details` (décision de l'orchestrateur)

- **Contexte** : validation P3 de la 35-2a, série 5 → 5 → 8 MEDIUM ; la moitié des MEDIUM de la P3 naissent du contrat
  public de `details` ajouté en P2 (V3-4 : onze raisons sans source unique ; V3-5 : noms de champs qui ne sont pas ceux de
  l'API ; V3-6 : une raison qui mêle trois causes). C'est le recyclage que vise la règle de découpage.
- **Retenu** : la 35-2a sert le besoin de Guy — le **message localisé** rendu par le serveur nomme le champ et le caractère,
  l'écran l'affiche —, **sans publier** de `details.reason` ni `details.field` ; `AppError::InvoiceNotPdfReady(String)` reste.
  Sont gardés : le pays nommé avec sa partie et son code, #964 (le champ vide nommé), le caractère échappé, la clé de la QRR.
  La 35-2b ne porte au lot que `details.message`, une phrase destinée à l'utilisateur. Aucun `details` n'existait sur ce code
  avant : rien n'est cassé.
- **Écarté** : fixer le contrat maintenant : public et irréversible, il ne se décide pas sous l'échéance du 30 novembre.
- **Suite** : **#1045**, « Contrat d'API lisible par machine pour INVOICE_NOT_PDF_READY » ; voisines : #1046 (jeu de
  caractères du format de numérotation), #1047 (avertissement au retypage d'un compte désigné, 35-3).
- **Réversible** : oui.

### C-e350-12 — 35-4 : *Rejeter* confirmé sur les seules lignes signalées, avec un texte propre au rejet

- **Contexte** : #1023 ; la garde de la 34-4 couvre *Accepter*, *Écriture manuelle* et *Éclater*
  (`ReconciliationProposals.svelte:288-291`), et l'intro du dialogue parle d'une écriture passée deux fois.
- **Retenu** : *Rejeter* passe par la même garde, **seulement** si la sélection contient une ligne signalée ; le dialogue
  reçoit le geste et dit ce que le rejet fait (rien n'est écrit ; un mouvement réel manquerait aux livres ; *Annuler le
  rejet* le défait). Il ne nomme pas l'écriture ressemblante : le serveur n'en donne que le nombre.
- **Écartées** : (a) confirmer tout rejet : friction sur le cas courant, sans signal ; (b) exposer l'écriture ressemblante :
  contrat d'API, hors d'une story d'écran.
- **Réversible** : oui.

### C-e350-13 — 35-4 : un geste confirmé sur une ligne partie affiche un avis, sans masquer la liste

- **Contexte** : #1024 et son commentaire : *Continuer quand même* sort en silence (`:347`, `:374`).
- **Retenu** : un avis au-dessus de la liste (« n'est plus à traiter … la liste est à jour »), effacé au geste suivant, pour
  les quatre gestes ; rien n'est envoyé. Pas par `errorMsg`, qui remplace la liste.
- **Réversible** : oui.

### C-e350-8, extension du 2026-11-12 — fichiers communs des stories 35-4, 35-5a, 35-5b, 35-6

- **`messages.ftl` ×4** : chaque story écrit ses clés **à côté de son bloc** : 35-4 près de
  `reconciliation-labels-already-booked-confirm-*` ; 35-6 près de `bank-accounts-opening-*` ; 35-5a/35-5b, s'il leur en
  faut, près de la clé qu'elles touchent. Jamais en fin de fichier.
- **`user-manual.tex`** : 35-4 `:3107` ; 35-6 `:2456` (et `:2462` si touché) ; 35-5a les phrases « aujourd'hui » du solde
  d'ouverture et du verrou de période ; 35-5b les autres. Paragraphes distincts ; le PDF se régénère après l'union.
- **`crates/kesh-api/src/routes/opening_balances.rs`** : 35-6 `parse_complement_amount` (`:515-530`) ; 35-5a la date du
  jour (`:244`, `:604`). Hunks distincts ; si les deux sont en vol, 35-6 fusionne d'abord (plus courte).
- **`crates/kesh-api/src/routes/bank_accounts.rs`** : 35-5a (`:1007`, `:1252`) ; 35-5b (`:1802`). 35-5b part de 35-5a fusionnée.
- **`docs/api-external.md`** : 35-5a la phrase « UTC » du solde d'ouverture ; 35-6 la grammaire du montant ; lignes
  distinctes.
- **`CHANGELOG.md`** : un point par issue sous `### Corrigé`, dans l'ordre des numéros de story.

### C-e350-15 — 35-6 : point et virgule ambigus, grammaire commune, #1025 intégrée

- **Contexte** : #1026 (`1.234` lu un franc en silence, `+1,234` non reconnu), #1028 (`1_000` accepté par le serveur,
  « NaN CHF » à l'écran), #1025 (`0,500` refusé à tort), toutes dans `format.ts` et `parse_complement_amount`.
- **Retenu** : un séparateur (point ou virgule) suivi de trois chiffres, partie entière de 1 à 3 chiffres sans zéro de
  tête, est ambigu et refusé avec la forme à écrire (`1'234` ou `1.2340`) ; une saisie hors grammaire est refusée à l'écran ;
  le serveur refuse le souligné et l'exposant, garde le point et le `+`. #1025 entre dans la story (même règle).
- **Écartées** : (a) signaler sans refuser `1.234` : la virgule est déjà refusée, deux comportements pour le même risque ;
  (b) juger l'ambiguïté au serveur : un appelant de l'API envoie un nombre, non une saisie.
- **Réversible** : oui ; le refus du souligné au serveur est un changement de contrat, écrit.

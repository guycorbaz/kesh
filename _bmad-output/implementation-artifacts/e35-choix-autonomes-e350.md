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
     *(Annoté le 2026-11-12, validation P5 de la 35-2a : **une** entrée seulement, `invoice_pdf_service.rs` (42 clés) — le
     repli de l'avoir est rendu par la fonction pure de ce fichier ; la table passe de 8 à **9**.)*
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
- *(Annoté le 2026-11-12, validation P1 de la 35-4 : **révisé par C-e350-18** — garde défensive, sans cause nommée.)*

### C-e350-8, extension du 2026-11-12 — fichiers communs des stories 35-4, 35-5a, 35-5b, 35-6

- **`messages.ftl` ×4** : chaque story écrit ses clés **à côté de son bloc** : 35-4 près de
  `reconciliation-labels-already-booked-confirm-*` ; 35-6 près de `bank-accounts-opening-*` ; 35-5a/35-5b, s'il leur en
  faut, près de la clé qu'elles touchent. Jamais en fin de fichier.
- **`user-manual.tex`** : 35-4 `:3107` ; 35-6 `:2456` (et `:2462` si touché) ; 35-5a les phrases « aujourd'hui » du solde
  d'ouverture et du verrou de période ; 35-5b les autres. Paragraphes distincts ; le PDF se régénère après l'union.
- **`crates/kesh-api/src/routes/opening_balances.rs`** : 35-6 `parse_complement_amount` (`:515-530`) ; 35-5a la date du
  jour (`:244`, `:604`). Hunks distincts ; si les deux sont en vol, 35-6 fusionne d'abord (plus courte).
  *(Annoté le 2026-11-12, validation P2 de la 35-6 : la 35-6 ne touche plus ce fichier, C-e350-19.)*
- **`crates/kesh-api/src/routes/bank_accounts.rs`** : 35-5a (`:1007`, `:1252`) ; 35-5b (`:1802`). 35-5b part de 35-5a fusionnée.
- **`docs/api-external.md`** : 35-5a la phrase « UTC » du solde d'ouverture ; 35-6 la grammaire du montant ; lignes
  distinctes. *(Annoté le 2026-11-12, validation P2 de la 35-5a : la 35-6 ne touche plus `api-external.md`, ni
  `opening_balances.rs` hors de son `mod tests` (un test de parité), C-e350-19 ; la 35-5b y écrit `:252`, `:292`, `:443`.)*
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
- *(Annoté le 2026-11-12, validation P1 de la 35-6 : précisé par C-e350-17 — classement avant le retrait des
  séparateurs, #1025 fermée en entier, le serveur garde `.5`, `5.` et `1.50000`.)*
- *(Annoté le 2026-11-12, validation P2 de la 35-6 : **remplacé par C-e350-19** pour le serveur — il n'est plus touché ;
  `1_000` reste accepté et l'écran le lit comme lui.)*

### C-e350-14 — La 35-5 se découpe préventivement en 35-5a et 35-5b (accord de l'orchestrateur)

- **Contexte** : #929, #1022, #907. L'inventaire de `release/0.17.6` compte **28 sites de production** qui prennent la date
  du jour en UTC, dans 17 fichiers de 4 crates (`kesh-api` routes et services, `kesh-db` dépôts, `kesh-seed`), plus 56 sites
  dans les modules de tests de `src`, 107 dans `crates/*/tests` et 36 dans les specs E2E. Plus de cinq modules : la règle
  de découpage préventif s'applique (`CLAUDE.md`, story-zéro puis rollout).
- **Retenu** : **35-5a** pose l'horloge, la fonction pure du jour suisse, la garde qui inventorie les sites restants (liste
  close d'exemptions), et convertit les sites qui **écrivent dans les livres ou les bornent** (solde d'ouverture,
  contre-passation, annulation de règlement, verrou de période : 8 sites) avec leurs tests. **35-5b** convertit les 20 sites
  restants et vide la liste d'exemptions, puis les specs E2E.
- **Écartée** : une seule story : boucle complète sur quatre crates et 200 sites de tests.
- **Réversible** : oui.
- *(Annoté le 2026-11-12, validations P1 des 35-5a et 35-5b : le site 17 passe à la 35-5a, 9 / 19 (C-e350-23) ; la 35-5b se
  découpe à son tour, la conversion des tests restants et la fermeture des gardes passant à une **35-5c** (C-e350-22) ;
  les décomptes « 36 » des specs E2E et « 107 » des tests valent 35 (34 + 1 coupé) et 112 (107 + 5 coupés).)*

### C-e350-16 — Le fuseau est Europe/Zurich, constant ; l'horloge s'injecte

- **Contexte** : il faut une date du jour « de l'installation ». Trois sources possibles : une constante, un réglage (variable
  d'environnement ou paramètre), le `TZ` du conteneur. L'image Docker tourne en UTC par défaut, et rien ne garantit que le
  `TZ` du conteneur de Guy soit posé ; l'écran a choisi le fuseau du poste (#889, `todayLocal`), réglé sur la Suisse.
- **Retenu** : **Europe/Zurich, constante unique** dans `kesh-core` (le jour civil suisse d'un instant UTC, par
  `chrono-tz`, déjà au `Cargo.lock` par `rxing`, version 0.10.4). Kesh est un logiciel de comptabilité suisse ; une
  installation hors de Suisse n'est pas un cas d'usage. Pas de lecture de `TZ` ni de `chrono::Local`.
- **Horloge injectable** : `kesh-core` porte la fonction **pure** `jour_suisse(instant: DateTime<Utc>) -> NaiveDate` ; `kesh-db`
  ne lit plus l'horloge : ses fonctions reçoivent la date du jour en paramètre ; `kesh-api` tient une horloge dans `AppState`
  (horloge système en production, horloge fixe dans les tests qui datent quelque chose), seule à appeler `Utc::now()` pour
  une date. Les tests ne dépendent plus de l'heure réelle : ils passent une date ou fixent l'horloge, et la nuit se teste
  par des instants fixés (00 h 30 le 1er janvier, heure d'hiver et d'été).
- **Écartées** : (a) variable `KESH_TIMEZONE` : une surface de configuration, une validation au démarrage et une ligne de
  manuel pour un besoin qui n'existe pas ; ajoutable plus tard sans casser la constante ; (b) `TZ` du conteneur : silencieux
  s'il manque (UTC), c'est le défaut même de #929.
- **Réversible** : oui ; un réglage peut remplacer la constante en un seul endroit.
- **Migration** : aucune ; rien ne change dans le schéma ni dans les données écrites, seule la date choisie pour les écritures
  à venir.

### C-e350-17 — 35-6 : classer avant de retirer les séparateurs ; rien d'accepté aujourd'hui n'est refusé sans être annoncé

- **Contexte** : validation P1 de la 35-6. La règle « une virgule suivie de trois chiffres » appliquée **après** le retrait
  des séparateurs laissait `1'234,567` ambigu (#1025) et donnait deux verdicts à une même valeur (`1234,567` / `1234.567`) ;
  `D3` aurait fait refuser au serveur `.5` et `5.`, acceptés aujourd'hui, sans l'annoncer.
- **Retenu** : la partie entière groupée (`1'234`) rend le séparateur suivant décimal ; non groupée, 1 à 3 chiffres sans
  zéro de tête suivis d'un séparateur et de trois chiffres : ambigu. `[0-9]` partout. Le serveur ne refuse de neuf que le
  souligné (et les chiffres non ASCII). L'écran normalise la charge (`.5` → `0.5`, sans `+`). #1025 fermée en entier.
  `opening_balances.rs:363` et `journal_entries.rs:542` (autres fonctions de lecture) restent à #1050 et #1051.
- *(Annoté le 2026-11-12, validation P2 de la 35-6 : la part serveur — refus du souligné et des chiffres non ASCII — est
  **retirée par C-e350-19** ; la règle d'écran reste.)*
- **Écartées** : (a) #1025 en partie : laisserait la moitié de l'issue ; (b) une grammaire serveur commune aux trois routes
  dès cette story : touche l'écriture manuelle, hors de l'objet.
- **Réversible** : oui.

### C-e350-18 — 35-4 : #1024 en garde défensive ; l'écriture ressemblante en issue ; `sitesTotal` à la règle d'union

- **Contexte** : validation P1 de la 35-4. Par lecture, la liste ne peut pas se relire pendant que la confirmation est ouverte
  dans un navigateur (modale bits-ui : focus piégé, clic hors = *Annuler* ; `load()` ne suit que `limit` et `bankAccountId`) ;
  le test qui fonde #1024 clique derrière la modale en jsdom ; le cas « autre onglet » est déjà rendu en `failed[]`.
  `AlreadyBooked` ne porte que des nombres. `i18n-keys.test.ts:972` (`sitesTotal`) est un compte exact que chaque story
  i18n déplace.
- **Retenu** : (1) l'avis de #1024 reste, en **garde défensive** testée en Vitest, sans nommer de cause, dans une région
  `role="status"` hors de la branche de la liste ; l'étape 0 constate dans un navigateur s'il existe un chemin réel ; sinon,
  ni capture ni description au manuel, et la PR le dit. (2) Nommer l'écriture ressemblante : issue à ouvrir. (3) La règle
  d'union C-e350-8 couvre `i18n-keys.test.ts` pour **toutes** les stories : chaque branche recompte `sitesTotal` à la
  source, l'intégration le recompte après chaque fusion, sans additionner.
- **Écartées** : (a) promettre un avis et une capture inatteignables ; (b) fermer #1024 sans rien changer : le silence reste
  possible sous un futur déclencheur de relecture.
- **Réversible** : oui.

### C-e350-19 — 35-6 : ce qui est accepté aujourd'hui reste accepté (principe posé par l'orchestrateur)

- **Contexte** : validation P2 de la 35-6, P1 5 → P2 5 MEDIUM, dont quatre nés de la grammaire écrite en P1 (contrôle du
  serveur, groupement fermé, `\d`) : la grammaire débordait de l'objet des issues.
- **Retenu** : ce qui est accepté aujourd'hui reste accepté avec la même valeur, **sauf** une saisie que Kesh lit avec une
  valeur différente de celle que l'utilisateur voulait (le cas « 1.234 ») ; chaque saisie nouvellement refusée est listée,
  justifiée et annoncée (CHANGELOG, manuel ; `api-external.md` avec « *Depuis la v0.17.7* » si l'API change). Pour la 35-6 :
  séparateurs d'aujourd'hui gardés, plus `_` (lu comme le serveur le lit) ; `-0` lu zéro ; vide inchangé ; **serveur non
  touché**, donc aucun changement de contrat ; nouvellement refusées : `1.234`, `12.500`, `999.999`, `1.005`, `+1.234`,
  `+1,234`. La story devient de l'écran seul : c'est la réponse au signal de découpage, plutôt qu'un découpage.
- **Écartée** : une grammaire fermée commune écran/serveur : changement de contrat sans demande, et refus nouveaux
  (`-0.00` d'un client d'API, groupements irréguliers) sans valeur pour Guy.
- **Réversible** : oui.
- **Commentaires déjà rédigés pour #1050 et #1051** (validation P1) : ils disaient que la 0.17.7 ferait refuser `1_000` au
  complément ; c'est faux depuis ce choix — correction à poster par l'orchestrateur.

*(C-e350-20 est pris par la 35-4, sur `chore/epic-35-kickoff` ; les entrées suivantes, de l'agent des fiches 35-5a à 35-5c
sur `chore/epic-35-fiches-355`, commencent à C-e350-21 et s'unissent à la fusion.)*

### C-e350-21 — 35-5 : l'horloge vit dans `AppState` et dans `Lecture`

- **Contexte** : validations P1 de la 35-5a (V1-6) et de la 35-5b (V1-1, HIGH). Une horloge dans le seul `AppState` n'atteint
  ni les services, qui ne reçoivent qu'une `Lecture` (`services/rapports.rs:726-732`, `:870`, `services/factures.rs:534`), ni
  le serveur MCP, qui construit sa propre `Lecture` au câblage (`lib.rs:1403`) et dont l'état ne voit pas l'`AppState`
  (`mcp/mod.rs:105-107`). Les fonctions pures (`InvoiceResponse::from_parts`, `routes/invoices.rs:333` ; la conversion d'un
  élément de liste, `:425` ; `parse_as_of`, `services/postes_ouverts.rs:61-74`) n'ont aucun contexte.
- **Retenu** : `Lecture` porte l'horloge — `Lecture::new(pool, horloge: Arc<dyn Horloge>)`, **sans valeur par défaut**,
  `Lecture::aujourd_hui()`. `AppState::lecture()` passe `self.horloge` ; le câblage du MCP, `state.horloge`. Les routes lisent
  `state.horloge.aujourd_hui()`, les services `lecture.aujourd_hui()` ; les fonctions pures, `kesh-db` et `kesh-seed` reçoivent
  une `NaiveDate`. Une lecture de l'horloge par requête. Posé par la 35-5a (qui en a besoin pour
  `services::factures::reglements`, rang 5) ; la 35-5b ne fait qu'en user. Trait `Horloge: Send + Sync` et `HorlogeFixe`
  (`pub`) dans `kesh-core` ; `HorlogeSysteme` dans `kesh-api`.
- **Coût** : 43 constructions `Lecture::new(` dans 10 fichiers de tests ; 16 littéraux d'`AppState` dans 9 fichiers ; la
  ligne figée par `mcp_gardes_lexicales.rs:169`, `:575` et `services_motif.rs` (`FABRIQUES_V2`, `:2071`, `:2083`) — changements
  mécaniques, que le compilateur et les gardes énumèrent.
- **Écartées** : (a) la date en paramètre de chaque service, et un champ d'`EtatMcp` / `ContexteMcp` pour le MCP : change la
  signature de chaque service servi par le MCP et de tous leurs appelants de tests (`services_rollout.rs`…), plus le contexte
  du MCP, pour le même effet ; (b) `Lecture::new(pool)` gardé, à horloge système par défaut, avec un second constructeur :
  laisse le MCP sur l'horloge système dans un test qui fixe celle de l'`AppState` (un appelant hors d'atteinte des tests), et
  ouvre un contournement de la garde (v-2), qui ne reconnaît que `Lecture::new(` ; (c) l'horloge dans le seul `AppState` :
  le défaut même de la validation.
- **Réversible** : oui ; une poignée et deux lignes de câblage.
- *(Annoté le 2026-11-12, validations P2 des 35-5a et 35-5b : les littéraux d'`AppState` sont **9 dans 7 fichiers**, non 16
  dans 9 ; le câblage du MCP s'écrit en trois lignes courtes (`horloge_outils`, `lecture_pool`, `lecture_outils`), la ligne
  d'un seul tenant dépassant 100 caractères ; `Lecture` expose aussi `horloge()`, pour le squelette de la 35-5b ; le service
  de l'échéancier rend la date qu'il a employée, que la route réemploie — une lecture par requête.)*

### C-e350-22 — La 35-5b se découpe : la 35-5c prend les tests restants et ferme les gardes

- **Contexte** : validation P1 de la 35-5b, V1-8. Après la plomberie de C-e350-21, la story portait 19 sites de production,
  le SQL de trois sites, la graine, le MCP, puis 168 sites de tests Rust et 35 de specs E2E, l'extension de deux gardes et la
  doc ; `kesh-db` touché impose le gate complet à chaque commit, sur un diff de plus de cent fichiers. Plus de cinq modules :
  la règle de découpage préventif s'applique une seconde fois.
- **Retenu** : **35-5b** = production, SQL, graine, plomberie des signatures, MCP, docs, **et** les tests dont l'attendu
  change avec un de ses sites (sinon le gate rougit à son propre commit) ; **35-5c** = les tests restants (`src`,
  `crates/*/tests`, specs E2E), la garde Rust fermée, la garde frontend étendue aux specs. **Chaque story reste sûre la
  nuit à elle seule** : la 35-5a convertit les tests qui atteignent ses sites (verdict des 38 fichiers dans sa fiche), la
  35-5b ceux des siens ; ce qui reste à la 35-5c ne compare pas son jour à celui du serveur. Ordre : 35-5a → 35-5b → 35-5c ;
  la **35-8** attend la 35-5c (`journal_entries.rs`, `mod tests`).
- **Écartées** : (a) 35-5b entière : une boucle d'environ une heure par passe sur plus de cent fichiers ; (b) la 35-5b emporte
  les 50 sites de `journal_entries.rs` pour libérer la 35-8 plus tôt : la 35-8 est en bas de l'ordre de l'epic, rien ne
  presse.
- **Réversible** : oui.

### C-e350-23 — 35-5a : la création du premier exercice par l'assistant passe à la 35-5a

- **Contexte** : validation P1 de la 35-5a, V1-5. `routes/onboarding.rs:1043-1056` crée « Exercice {année} » par
  `fiscal_years::create_if_absent_in_tx`, ligne de `fiscal_years` tracée à l'audit : la nuit du 1er janvier, l'exercice de
  l'année passée, et les soldes de départ datés du 1er janvier de cette année-là. C-e350-14 confie à la 35-5a « ce qui écrit
  dans les livres ou les borne ».
- **Retenu** : le site 17 passe à la 35-5a (AC5 de sa fiche) ; partage 9 / 19.
- **Écartée** : le laisser à la 35-5b en écrivant pourquoi un exercice n'est pas « dans les livres » : il l'est (il borne
  toutes les écritures).
- **Réversible** : oui.

### C-e350-24 — 35-5 : la garde de la date du jour, par fichier, en égalité stricte, sur les sources et les tests

- **Contexte** : validations P1 de la 35-5a (V1-3, V1-4) et de la 35-5b (V1-4). La garde écrite au kickoff comptait les
  commentaires (8 dans `src`), ne disait pas ce qu'est un compte « qui ne baisse pas comme prévu », énumérait des formes,
  ne voyait pas les 5 sites coupés sur plusieurs lignes, et ne lisait pas `crates/*/tests`, où un oubli ne rougit que la
  nuit.
- **Retenu** : `crates/kesh-api/tests/date_du_jour_garde.rs`, dès la 35-5a, sur `crates/*/src/**/*.rs` **et**
  `crates/*/tests/**/*.rs` ; lignes de commentaire (`//`, `///`, `//!` en tête) écartées, blancs réduits autour des `.` d'une
  chaîne de méthodes (formes coupées vues) ; formes relevées sur **tout récepteur** (`.date_naive()`, `.naive_utc().date()`,
  `.naive_local()`, `Utc::today`, `Local::`, `UTC_DATE`, `CURDATE`, `CURRENT_DATE`, `DATE(NOW`) ; liste d'exemptions
  `{ fichier, nombre, story, motif }` en **égalité stricte** — un fichier absent compte 0, un écart dans un sens ou dans
  l'autre rougit, la liste se ramène au compte réel à chaque conversion. Entrée permanente : `jour_suisse` (`kesh-core`, 1).
  La 35-5c la ferme. Côté écran, `local-date-guard.test.ts` s'étend aux specs à la 35-5c.
- **Écartées** : (a) un compte « décroissant » (`<=`) : ne voit pas une exemption gardée après conversion (M5) ; (b) des
  commentaires comptés, comme `local-date-guard.test.ts` : rougirait pour des phrases vraies (`aged_receivables.rs:27`) ; (c)
  une garde restreinte à `src` jusqu'à la 35-5b : un test converti pourrait revenir en arrière sans bruit.
- **Réversible** : oui.
- *(Annoté le 2026-11-12, validation P2 de la 35-5a et P1 de la 35-5c : motifs bornés `\b…\(` (le relevé « casse ignorée »
  de `UTC_DATE` prenait `Utc.from_utc_datetime(`) ; formes `Utc::now().year(`/`month(`/`day(`/`format(` et
  `DATE(UTC_TIMESTAMP`/`DATE(CURRENT_TIMESTAMP` ajoutées ; entrée permanente `admin.rs:722` (horodatage) ; le fichier de la
  garde est exclu nommément, une sonde le prouve ; angle mort assumé : le jour tiré d'une variable d'instant.)*

### C-e350-26 — 35-5 : la preuve de nuit par `KESH_TEST_INSTANT`

- **Contexte** : validation P1 de la 35-5c (V1-7) — aucun critère n'établissait qu'un test converti passe « à 00 h 30 » ; les
  tests de montage prennent `aujourd_hui_suisse()`, qui lit l'horloge réelle. `faketime` n'est pas installé sur la station
  (`command -v faketime` vide, aucune `libfaketime`).
- **Retenu** : `AppState::new_for_tests` et les helpers `aujourd_hui_suisse()` prennent une `HorlogeFixe` à l'instant de la
  variable `KESH_TEST_INSTANT` (RFC 3339) si elle est posée, l'horloge système sinon ; lue à chaque appel, sans `static` ni
  `OnceLock`. `main.rs` ne la lit jamais (il construit `HorlogeSysteme`), et la garde de la date du jour vérifie qu'elle
  n'apparaît que dans ces trois fonctions. Chaque story (35-5a, 35-5b, 35-5c) lance une fois les binaires de tests qu'elle
  convertit à `2025-12-31T23:30:00Z` et à `2026-03-10T23:30:00Z` ; journal `nuit.log` ; un rouge qui vient d'un horodatage
  MariaDB (`NOW(3)`, heure réelle) se nomme, il n'est pas un défaut de nuit. Posé par la 35-5a.
- **Écartées** : (a) installer `faketime` : un paquet sur la station, et un `LD_PRELOAD` sous `cargo test` peu sûr ; (b) une
  heure de nuit fixée dans chaque test : 112 sites réécrits pour une preuve qu'une variable donne ; (c) ne rien prouver : la
  règle « on ne déclare que ce qui a tourné ».
- **Réversible** : oui ; trois fonctions de test.

### C-e350-27 — 35-5a : dérogation à la règle de découpage préventif

- **Contexte** : validation P2 de la 35-5a, V2-8 : la story touche plus de cinq modules.
- **Retenu** : pas de nouveau découpage ; section *Dérogation règle de splitting* dans la fiche. Le mécanisme ne se coupe
  pas — les signatures de D2 et de `Lecture::new` obligent tous leurs appelants, production et tests, dans le même commit — et
  la sûreté de nuit non plus (une story qui convertirait les sites sans les tests qui les atteignent rougirait la nuit). Ce
  qui pouvait partir est parti : 35-5b (sites restants), 35-5c (tests sans lien), la dévalidation (V2-10).
- **Écartée** : isoler les appelants sans site de jour dans la 35-5c : la compilation de la 35-5a l'interdit.
- **Réversible** : oui.

### C-e350-25 — 35-5 : le jour d'un instant enregistré

- **Contexte** : validations P1 de la 35-5a (V1-7) et de la 35-5b (V1-2). Des horodatages UTC sont ramenés à un jour par
  `.date()` ou `DATE(…)` : `dunning_eligibility.rs:128` (`lr.date()`, comparé au jour du serveur : une fois ce jour suisse,
  le niveau suivant d'un rappel envoyé entre minuit et 1 h devient éligible un jour trop tôt), `routes/invoices.rs:1525`
  (`paid_at` du CSV de l'échéancier), `reconciliation.rs:237` (`DATE(rmd.last_sent_at)` dans la date d'exigibilité, en
  SQL), `reconciliation.rs:435` (contrainte `chk_invoices_paid_at_after_date`, `DATE(paid_at) >= date - INTERVAL 1 DAY`).
- **Retenu** : `dunning_eligibility.rs:128` et `routes/invoices.rs:1525` passent à `jour_suisse(x.and_utc())` dans la 35-5b
  (sites 22 bis et 16 bis) ; `reconciliation.rs:237` s'ouvre en issue (SQL : le jour suisse d'un horodatage demande soit les
  tables de fuseaux de MariaDB, soit un calcul hors SQL ; hors de la limite courte) ; `:435` reste tel quel, sans issue : la
  tolérance d'un jour absorbe l'écart, et la changer demanderait une migration (règle d'arrêt).
- **Écartée** : laisser toute la classe en UTC : le mélange des deux horloges dans une même comparaison (`:128`) est le
  défaut contre lequel `companies.rs:986-989` met en garde.
- **Réversible** : oui.

### C-e350-8, extension du 2026-11-12 (suite) — 35-3 et 35-5a

- `kesh-db/src/repositories/supplier_invoices.rs` (35-3 `:1521-1530`, `:1698-1727` ; 35-5a, signatures `:2462`, `:2600`,
  `:2750`, `:2871`), `invoice_settlements_write.rs` (35-3 `:679-680` ; 35-5a `:878`, `:1023`),
  `crates/kesh-db/tests/supplier_invoices_repository.rs` (35-3 `:2250-2257` ; 35-5a, ses dates du jour) : hunks distincts ; la
  seconde fusionnée se rebase sur la première et recompile (la 35-5a ajoute un argument aux fonctions que la 35-3 appelle
  peut-être). `user-manual.tex` : 35-5a et 35-6 aussi, paragraphes distincts. `routes/invoices.rs`,
  `routes/supplier_invoices.rs`, `routes/reconciliation.rs` : 35-5a (chaîne des appelants), 35-5b (`routes/invoices.rs`).

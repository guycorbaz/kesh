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
- *(Annoté le 2026-11-12, validation P2 de la 35-4 : **remplacé par C-e350-20** — plus d'avis.)*

### C-e350-8, extension du 2026-11-12 — fichiers communs des stories 35-4, 35-5a, 35-5b, 35-6

- **`messages.ftl` ×4** : chaque story écrit ses clés **à côté de son bloc** : 35-4 près de
  `reconciliation-labels-already-booked-confirm-*` ; 35-6 près de `bank-accounts-opening-*` ; 35-5a/35-5b, s'il leur en
  faut, près de la clé qu'elles touchent. Jamais en fin de fichier.
- **`user-manual.tex`** : 35-4 `:3049-3050`, `:3097`, `:3107` *(`:3097` ajouté le 2026-11-12, validation P2 de la 35-4 ;
  `:3049-3050` le même jour, validation P3)* ; 35-6 `:2456` (et `:2462` si touché) ; 35-5a les phrases « aujourd'hui » du solde
  d'ouverture et du verrou de période ; 35-5b les autres. Paragraphes distincts ; le PDF se régénère après l'union.
- **`crates/kesh-api/src/routes/opening_balances.rs`** : 35-6 `parse_complement_amount` (`:515-530`) ; 35-5a la date du
  jour (`:244`, `:604`). Hunks distincts ; si les deux sont en vol, 35-6 fusionne d'abord (plus courte).
  *(Annoté le 2026-11-12, validation P2 de la 35-6 : la 35-6 ne touche plus ce fichier, C-e350-19.)*
  *(Annoté le 2026-11-12, validation P3 de la 35-6 : la 35-6 y ajoute **un test seul**, au `mod tests` (`:630` et suivantes),
  le test de parité de C-e350-33 ; aucun hunk de production ; hunk distinct de ceux de la 35-5a.)*
- **`crates/kesh-api/src/routes/bank_accounts.rs`** : 35-5a (`:1007`, `:1252`) ; 35-5b (`:1802`). 35-5b part de 35-5a fusionnée.
- **`docs/api-external.md`** : 35-5a la phrase « UTC » du solde d'ouverture ; 35-6 la grammaire du montant ; lignes
  distinctes.
  *(Annoté le 2026-11-12, validation P3 de la 35-6 : la 35-6 ne touche plus ce fichier, C-e350-19 ; seule la 35-5a y écrit.)*
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
- *(Annoté le 2026-11-12, validation P2 de la 35-4 : le point (1) est **remplacé par C-e350-20**.)*
- *(Annoté le 2026-11-12, validation P3 de la 35-4 : « le cas « autre onglet » est déjà rendu en `failed[]` » ne vaut que
  pour *Accepter* et *Rejeter* ; *Écriture manuelle* et *Éclater* sont unitaires — refus d'une ligne rapprochée dans leur
  fenêtre, ligne rejetée ailleurs acceptée et son rejet levé. Détail : fiche 35-4, « Ce qui se lit au code ».)*
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
- *(Annoté le 2026-11-12, validation P3 de la 35-6 : les six saisies « nouvellement refusées » ci-dessus n'étaient pas un
  ensemble fini mais des exemples ; la règle décidable et ses trois familles sont **C-e350-30** et **C-e350-32**.)*

### C-e350-20 — 35-4 : une relecture de la liste referme la confirmation (décision demandée par l'orchestrateur)

- **Contexte** : validation P2 de la 35-4, V2-2. La cause du silence de #1024 est que `load()` ne referme pas la
  confirmation : le dialogue, dans la branche de la liste, est démonté puis remonté **ouvert** sur des transactions périmées.
  L'avis de C-e350-18 ne venait qu'après le clic.
- **Retenu** : `load()` pose `pendingGesture = null` : une confirmation ouverte sur des lignes périmées ne peut jamais être
  validée. Plus d'avis, de clé ni de région de statut. La PR porte `refs #1024` ; l'orchestrateur ferme l'issue à la main
  avec le commentaire de la fiche (D5).
- **Écartée** : l'avis après le clic : plus de code, une confirmation périmée encore affichée, un texte sans cause.
- **Réversible** : oui.

*(C-e350-21 à C-e350-29 : réservés à la branche `chore/epic-35-fiches-355` ; l'union les range à leur place.)*

### C-e350-30 — 35-6 : la lecture en pipeline ordonné, et le refus par règle décidable

- **Contexte** : validation P3 de la 35-6 (V3-1, V3-2, V3-3). D2 ne disait ni le rognage ni l'ordre des opérations ; classée
  sur la saisie brute, `1,23 4` aurait été lue 1,234 franc en silence, et ` 9,999 ` aurait échappé à l'ambiguïté. D4 annonçait
  une « liste close » de six saisies, alors que la règle refuse une famille (`1.000`, `100.250`…).
- **Retenu** : un pipeline L1-L7 (rognage ; un signe en tête, reste rogné ; souligné ; au plus un `.`/`,` ; fraction en
  chiffres seuls ; partie entière groupée ou non ; classement non lisible → négative → ambiguë → claire). Ambiguë ⇔
  `^0*[1-9][0-9]{0,2}[.,][0-9]{3}$` sur la partie entière non groupée et la fraction telle que saisie. Trois familles
  nouvellement refusées, annoncées par la règle : A (le point), B (la virgule masquée par un `+`), C (blanc ou apostrophe
  après la décimale). Clé neuve `bank-accounts-opening-error-amount-unreadable`.
- **Écartées** : (a) retirer les séparateurs partout puis classer : rouvre `1'234,567` (#1025) ; (b) retirer les blancs de la
  fraction : laisse `1. 234` lu 1,234 franc ; (c) annoncer une liste d'exemples : trompe sur l'étendue du refus.
- **Réversible** : oui.
- *(Annoté le 2026-11-12, validation P4 de la 35-6 : B élargie au `_`, L1 qui rogne aussi les apostrophes, familles D et E
  ajoutées — **C-e350-34** et **C-e350-35**.)*

### C-e350-31 — 35-6 : le souligné suit la règle du serveur, non « entre deux chiffres »

- **Contexte** : validation P3 de la 35-6, V3-1, proposait de ne retirer `_` qu'entre deux chiffres. `rust_decimal` 1.43.0
  (`str.rs:245`, `b'_' if HAS`) l'accepte dès qu'un chiffre a été lu : `1_` vaut 1, `1_.5` vaut 1,5, aujourd'hui acceptées.
- **Retenu** : `_` avant le premier chiffre → non lisible (comme le serveur, `str.rs:325`) ; ailleurs retiré, transparent pour
  le classement (`1_.234` est ambiguë comme `1.234`).
- **Écartée** : « entre deux chiffres » : refuse `1_` et `1_.5` sans qu'elles soient mal lues, contraire à C-e350-19.
- **Réversible** : oui.

### C-e350-32 — 35-6 : les zéros de tête ne comptent pas dans la partie entière courte

- **Contexte** : validation P3 de la 35-6, V3-4. `01,234`, `012,345`, `00,500` sont refusées aujourd'hui (`^\d+,\d{3}$`) ; la
  règle sans zéro de tête les aurait rendues claires (`01,234` lue 1,234 franc, en silence) ; `01.234` est acceptée
  aujourd'hui, lue 1,234.
- **Retenu** : zéros de tête ôtés avant le compte de 1 à 3 chiffres, le zéro seul exclu : `01,234`, `012,345` restent
  ambiguës ; `00,500` devient claire (0,5, comme `0,500`) ; `01.234` est **nouvellement refusée** (famille A) : même risque de
  lecture que `1.234`.
- **Écartée** : garder `01.234` claire : deux verdicts pour `01,234` et `01.234`, et une règle à deux branches.
- **Réversible** : oui.

### C-e350-33 — 35-6 : un test de parité écran/serveur sur une fixture commune

- **Contexte** : validation P3 de la 35-6, V3-9 : « l'écran lit `1_000` comme le serveur » n'était prouvé par aucun test.
- **Retenu** : la table de D3 vit dans une fixture JSON (`frontend/src/lib/features/bank-accounts/opening-amount-cases.json`),
  lue par Vitest et, à l'exécution, par un test du `mod tests` de `opening_balances.rs` qui confronte
  `parse_complement_amount` aux charges et aux saisies brutes de l'alphabet du serveur. Aucune ligne de production du serveur.
  Fichier commun avec la 35-5a (C-e350-8, annoté) : hunk de test distinct.
- **Écartées** : (a) reformuler sans tester : l'affirmation reste crue sur parole ; (b) recopier la table en Rust : deux
  tables qui dérivent ; (c) `include_str!` : lierait la compilation de `kesh-api` à l'arborescence du frontend.
- **Réversible** : oui.

### C-e350-34 — 35-6 : la notation à exposant refusée à l'écran seulement (décision de l'orchestrateur)

- **Contexte** : validation P4 de la 35-6, V4-1. `Decimal::from_str` se rabat sur `from_scientific_lossy` quand la saisie porte
  un `e` ou un `E` (`rust_decimal` 1.43.0 `decimal.rs:2108-2117`) : `1e3` vaut 1000, accepté aujourd'hui par le serveur ;
  l'écran l'envoie tel quel. La fiche le disait refusé.
- **Retenu** : **famille D** nouvellement refusée **par l'écran seulement** (non lisible) ; le serveur, inchangé, l'accepte
  toujours pour un client d'API. **Écart assumé au principe C-e350-19**, motivé par l'improbabilité d'une telle saisie à
  l'écran et le risque d'une lecture inattendue (l'aperçu ne l'a jamais montrée ; `1.234e0` échapperait à la règle
  d'ambiguïté). Annoncée au CHANGELOG et au manuel ; ligne de table ; le test de parité épingle la lecture du serveur
  (AC8 (e)).
- **Écartée** : accepter l'exposant à l'écran : une règle d'ambiguïté de plus, sur la mantisse, pour une saisie que personne
  ne fait.
- **Réversible** : oui.

### C-e350-35 — 35-6 : la virgule masquée par un `_`, les apostrophes de bord, plus de 28 décimales

- **Contexte** : validation P4 de la 35-6, V4-2. Le balayage de la passe trouve, acceptées aujourd'hui et refusées par le
  pipeline hors des familles écrites : `1_,234` (le `_` cachait la virgule à l'écran ; lue 1,234), `'+5` et `'-0`
  (l'apostrophe était retirée partout), et plus de 28 décimales (le serveur arrondit).
- **Retenu** : (1) famille B élargie au `_` : ambiguë, nouvellement refusée ; (2) L1 et la reprise de L2 rognent aussi les
  apostrophes de tête et de queue : `'+5`, `'-0`, `1.5'` restent acceptées avec leur valeur ; (3) **famille E**, plus de
  28 décimales dont l'une au-delà de la quatrième n'est pas nulle : non lisible, limite assumée, dite au CHANGELOG.
- **Écartées** : (a) une famille « apostrophe avant le signe » : refuserait sans raison une saisie lue juste ;
  (b) arrondir comme le serveur au-delà de 28 décimales : une règle pour un cas pathologique.
- **Réversible** : oui.

### C-e350-36 — 35-7 : un compte que le dialogue ne propose plus est retiré de la sélection

- **Contexte** : #1000, chemin 1. Dans *Écriture manuelle* et *Éclater*, le compte de la banque, choisi avant que son
  identifiant soit résolu, sort ensuite de la liste : le champ paraît vide, l'identifiant part au serveur, refusé en 400.
- **Retenu** : une fonction pure `keepOfferedCounterparty` (`reconciliation-counterparties.ts`) remet la sélection à `null`
  quand le compte, connu de la liste complète, n'est plus proposé ; un compte inconnu (liste pas chargée) est gardé, le
  serveur tranche. Le message existant « Compte de contrepartie obligatoire » s'applique ; aucun texte neuf.
- **Écartée** : afficher le compte retenu hors liste, marqué : un compte que le dialogue refuse resterait sélectionnable
  en apparence, et le serveur le refuserait de toute façon.
- **Réversible** : oui.

### C-e350-37 — 35-7 : sur un champ de compte obligatoire, quitter le champ jette le brouillon

- **Contexte** : #1000, chemin 2. Sans `allowClear`, `AccountAutocomplete` garde au `blur` le texte tapé (vide ou libre)
  pendant que la valeur reste l'ancien compte : un compte invisible part au serveur.
- **Retenu** : au `blur`, sans `allowClear` et hors mode dégradé, le brouillon est jeté : le champ réaffiche le compte
  retenu, sans jamais appeler `onSelect` (#271 tenue : un champ obligatoire n'est pas nullifié au `blur`).
- **Écartées** : (a) nullifier au `blur` : rouvre #271 sur les lignes d'écriture ; (b) garder le brouillon et bloquer la
  validation : un nouveau message et un nouvel état pour cinq consommateurs.
- **Réversible** : oui.

### C-e350-38 — 35-8 : découpage préventif en 35-8a (motif et pilotes) et 35-8b (déploiement)

- **Contexte** : #431. Sur `main`, 78 sites `NewAuditLogEntry::user(` de production dans 29 fichiers, 9 `letterings::Actor`
  sans clé, 3 `None` positionnels ; seize dépôts de `kesh-db` et autant de routes. Plus de cinq modules : la règle de
  découpage préventif s'applique avant `bmad-create-story`.
- **Retenu** : **35-8a** pose `Acteur`, `NewAuditLogEntry::par`, le pont depuis `CurrentUser`, la garde des sites non
  résolus, et convertit deux pilotes (les écritures ; le retour en brouillon d'une facture fournisseur, #892 item 11) ;
  **35-8b** convertit le reste et vide la raison `A_CONVERTIR_35_8B` de la garde. La clé de sprint `35-8` passe à `split`.
- **Écartée** : une seule story : une boucle de revue sur seize dépôts.
- **Réversible** : oui.

### C-e350-39 — 35-8 : un type `Acteur` dans `kesh-db`, à la place du `user_id` nu

- **Contexte** : passer `(user_id, actor_api_key_id)` laisse un `None` positionnel invisible (`invoices.rs:1689`,
  `supplier_invoices.rs:1410`, `:1412`) ; `letterings::Actor` existe déjà.
- **Retenu** : `Acteur { user_id, api_key_id }` dans `entities/audit_log.rs`, `NewAuditLogEntry::par`, un seul constructeur
  sans clé (`Acteur::utilisateur`, greppable), `From<&CurrentUser>` dans `kesh-api` ; `letterings::Actor` en devient l'alias
  ou disparaît. Une fonction convertie prend l'`Acteur` à la place du `user_id`.
- **Écartées** : (a) ajouter un paramètre `Option<i64>` partout : le `None` positionnel reste muet ; (b) déplacer
  `CurrentUser` dans `kesh-db` : un type d'authentification dans la couche de données.
- **Réversible** : oui (aucun schéma touché).

### C-e350-40 — 35-8 : une garde qui inventorie les sites non résolus, par raison

- **Contexte** : `CLAUDE.md`, § *Inventorier les sites NON RÉSOLUS* : énumérer les sites convertis laisse passer le
  suivant.
- **Retenu** : `crates/kesh-api/tests/audit_acteur_garde.rs` compte par fichier `NewAuditLogEntry::user(`,
  `Acteur::utilisateur(` et `api_key_id: None` dans le code de production, contre une table close à raisons
  (`PRE_AUTH`, `ADMIN_SANS_CLE` avec l'appelant qui le prouve, `MCP`, `PONT`, `A_CONVERTIR_35_8B`). Le serveur MCP reste
  hors de #431 : une clé MCP n'est pas une clé REST, et le manuel écrit déjà la limite (`admin-manual.tex:3392`).
- **Écartée** : un test par route : couvre les routes écrites, non celles qui viendront.
- **Réversible** : oui.

### C-e350-41 — 35-8b : l'`Acteur` partout où un handler authentifié trace, administration comprise

- **Contexte** : 35-8b. Exempter les dépôts appelés « seulement par l'administration » oblige à prouver, pour chacun, que
  nulle route comptable ne l'appelle, et fait reposer la piste de contrôle sur la place de `require_not_pat`.
- **Retenu** : tout site atteint par un handler authentifié prend l'`Acteur` de la requête ; restent hors du motif le
  pré-authentification, le système (réparation au démarrage), le pont et le serveur MCP. Les raisons `A_CONVERTIR_35_8B`
  et `ADMIN_SANS_CLE` disparaissent de la garde.
- **Écartée** : garder `ADMIN_SANS_CLE` : moins de sites à convertir, mais une preuve d'appelant à tenir à jour.
- **Réversible** : oui.

### C-e350-42 — 35-1 : un format de facture ou d'avoir doit porter un compteur

- **Contexte** : #932, #948. `validate_template` accepte `F-{YEAR}`, qui donne le même numéro à toutes les factures d'un
  exercice.
- **Retenu** : variante `FormatError::NoSeq` ; `NoPlaceholder` garde son sens et passe avant. Miroir d'écran.
- **Écartée** : la forme stricte du format fournisseur (`validate_numbering_template`) : refuserait `{FY}/{SEQ:06}` et
  d'autres formats valides aujourd'hui.
- **Réversible** : oui.

### C-e350-43 — 35-1 : un format n'est jugé que s'il change

- **Contexte** : #932 : durcir la validation bloquerait l'enregistrement de la page sur un format d'avoir que l'écran ne
  montre pas.
- **Retenu** : la route ne juge un format que si la requête le porte et qu'il diffère de la valeur enregistrée ; aucune
  migration ne réécrit un format enregistré.
- **Écartée** : réécrire les formats sans compteur par migration : une migration, donc l'arrêt de l'epic (C-e350-5), pour un
  cas que Guy n'a pas.
- **Réversible** : oui.

### C-e350-44 — 35-1 : le format des avoirs devient visible à l'écran

- **Contexte** : le formulaire n'envoie pas `creditNoteNumberFormat` ; le manuel (`user-manual.tex:1561`) le dit pourtant
  « configurable dans Administration → Facturation ».
- **Retenu** : un champ *Format des avoirs*, avec aperçu, dans la section *Numérotation*.
- **Écartée** : laisser le format d'avoir à l'API : un format fautif enregistré ne se corrigerait pas à l'écran.
- **Réversible** : oui.

### C-e350-45 — 35-1 : un numéro impossible à produire est un refus typé, non une erreur 500

- **Contexte** : `render` échoue sur un rendu > 64 (un nom d'exercice va jusqu'à 50 caractères, compté 16) ou une
  accolade dans le nom d'exercice ; `invoices.rs:2334-2340` et `credit_notes.rs:572-577` en font un `Invariant`, 500.
- **Retenu** : `DbError::NumberFormatUnrenderable`, code `NUMBER_FORMAT_UNRENDERABLE`, même statut que
  `DesignatedAccountsNotPostable`, message i18n qui nomme le format, l'exercice et la cause ; rien d'écrit.
- **Écartée** : compter `{FY}` pour 50 à la validation du format : refuserait des formats valides aujourd'hui.
- **Réversible** : oui.

### C-e350-46 — 35-2b : le message du refus pour les trois causes du code, par une construction commune

- **Contexte** : validation P1 de la 35-2b, V1-1. `classify_render_error` rend `INVOICE_NOT_PDF_READY` pour
  `InvoiceNotPdfReady`, `InvoiceTooManyLinesForPdf` et `InvoicePdfHeaderOverflow` ; la fiche ne donnait `details.message`
  qu'à la première, alors que la PR ferme #955. Les deux autres ont une phrase sur la route unitaire, construite dans le bras
  de `IntoResponse` (`errors.rs:2144-2168`).
- **Retenu** : voie (a) de l'orchestrateur — deux fonctions `pub(crate)` extraites d'`errors.rs`
  (`message_trop_de_lignes_pdf`, `message_entete_pdf_deborde`), appelées par `IntoResponse` et par le lot : une seule source
  par phrase ; `details.message` pour les trois causes. Simple : deux fonctions sans état, statut et code inchangés.
- **Écartée** : voie (b) — limite écrite, issue, `refs #955` : laisse le lot muet pour deux causes que l'utilisateur peut
  corriger.
- **Réversible** : oui.

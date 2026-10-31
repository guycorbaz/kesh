# Story v016-2b : saisir un relevé bancaire à la main — l'écran

Status: review

*Sous-fiche issue du découpage de la Story v016-2 à la validation P2 (2026-10-30, décision de l'orchestrateur,
C-v016-2-18). **Dépend de la `v016-2a`** (l'API, les codes, l'audit, la migration) : elle se développe après sa fusion
sur la branche. Worktree `/home/gcorbaz/devel/kesh-0162`, branche `story/v016-2-releve-manuel` (partie de `c5101b07`), la 2a d'abord,
puis la 2b. Registre **commun** : `v016-2-choix-autonomes.md` (C-v016-2-1 à C-v016-2-20 ; borne portée à la validation P7, P4). Issue : **#816**, en `refs`
seulement ; la fermeture se porte sur la PR de release. Les numéros de ligne cités sont ceux de `c5101b07`.
**Contexte, décisions, contrat de l'API (corps, réponse de l'aperçu, codes d'erreur, ensembles fermés, ordre des
refus), limites, liste de la release et questions au Project Lead : fiche parente `v016-2-releve-manuel.md`** — ils
valent ici sans être recopiés. Les AC gardent leur numéro d'origine (table de correspondance de la parente).*

## Story

En tant que Guy, je veux **saisir à l'écran** le relevé d'un compte sans relevé électronique — période, soldes, lignes au
clavier, écart affiché en continu —, le retrouver dans la liste, le rapprocher, et lire au manuel quand employer cette
voie et comment s'en sortir si la banque passe un jour au CAMT.

## Périmètre

Le frontend (`features/bank-import`, la page `/bank-import/manual`, la liste, le détail, la boîte de suppression, la
généralisation d'`isValidAmount` dans `features/journal-entries`), les clés de l'écran aux quatre catalogues, le manuel
utilisateur (sous-section, inventaire des phrases, FAQ) et `admin-manual.tex:82`, l'E2E Playwright, les captures,
l'entrée *Ajouté* du CHANGELOG et la ligne *Fonctionnalités* du README.

## Critères d'acceptation

- **AC-19** — `/bank-import` porte, pour un Comptable ou un Administrateur, un lien **« Saisir un relevé à la main »**
  (`data-testid="bank-import-manual-link"`) vers `/bank-import/manual` ; un utilisateur Consultation ne le voit pas, et
  `/bank-import/manual` le renvoie comme le font les autres écrans de saisie (patron du dépôt, à citer).
- **AC-20** — Le formulaire `ManualStatementForm.svelte` (`frontend/src/lib/features/bank-import/`) : compte (comptes
  de `GET /companies/current`, comme le dépôt), période (du, au), numéro (facultatif), solde de départ, solde d'arrivée,
  puis la table des lignes — Date, Valeur, Libellé, Contrepartie, Référence, **Débit**, **Crédit** — avec *Ajouter une
  ligne* et un bouton de retrait par ligne. Une ligne porte exactement un des deux montants. Le formulaire s'ouvre avec
  **une** ligne vide ; une ligne entièrement vide — chaque champ vide **après `trim`** (validation P3, R6 c) — est
  retirée à l'envoi, et le formulaire **garde la correspondance**
  entre le rang envoyé et la ligne affichée : tout rang rendu par le serveur (`line` d'une erreur, `line` d'un doublon)
  se traduit en ligne de l'écran avant d'être affiché (validation P1, R10/F5). Vitest : une ligne vide au milieu, une
  faute ou un doublon sur la suivante → c'est **cette** ligne qui est désignée.
- **AC-21** — Sous la table, mis à jour à chaque frappe, par une fonction pure `manual-statement.ts` (`big.js`, jamais
  `parseFloat`) : total des débits, total des crédits, solde calculé (`départ + crédits − débits`), solde d'arrivée
  saisi, **écart** (`arrivée − calculé`), au format suisse ; un solde vide n'est **pas** zéro : l'écart ne s'affiche
  pas tant que les deux soldes ne sont pas saisis. *Enregistrer* est **inactif** tant que : l'écart n'est pas nul ; un
  solde est vide ou invalide ; une ligne non vide viole une règle de l'AC-4 vue côté client — date de comptabilisation
  vide, illisible ou hors période (la date de valeur est libre), libellé vide ou de plus de 500 caractères,
  contrepartie, référence ou numéro de plus de 255 caractères, zéro ou deux montants, montant nul ou hors forme — ;
  la période est inversée ou les lignes dépassent 500 ; l'aperçu n'a pas **réussi** sur l'état courant du formulaire
  (un aperçu qui a rendu `BANK_STATEMENT_INVALID` ou `_UNBALANCED` ne compte pas) ;
  ou une case de confirmation **exigée** par l'aperçu n'est pas cochée ; et (validation P3, R2) aucun compte n'est
  choisi, ou une borne de la période est vide ou illisible — « hors période » n'est alors pas évalué. La raison est dite
  à côté du bouton : **une seule**, la première dans cette séquence *(récrite seule et complète à la validation P4,
  P6)* : compte ; période (vide, illisible, inversée) ; numéro du relevé ; soldes ; nombre de lignes ; première ligne
  fautive ; écart ; aperçu ; cases. *Vérifier* est actif dès qu'un compte et
  une période lisible sont saisis, même si l'écart n'est pas nul ou qu'une ligne est fautive — si bien que les erreurs
  du serveur de l'AC-22 sont atteintes par l'usage normal. Pour M14, le test pose un **aperçu réussi** sur un formulaire
  déséquilibré et attend la raison « écart ». *(Validation P3, R6 :)* la date de valeur n'est pas contrôlée par le
  client (un `<input type="date">` la rend lisible ou vide) ; une faute de serveur désigne **la cellule** (ligne et
  colonne, `field` traduit par l'en-tête de colonne déjà affiché) et le message est le libellé du `reason` ; un `field`
  d'**entête** (`line` nul : période, numéro, soldes, `lines`) désigne le champ du formulaire, traduit par son libellé
  (validation P4, P5 ; Vitest). *Asymétrie de taille* (validation P4, P4) : le client admet 500 lignes pleines
  (≈ 2,1 Mo), le serveur refuse un corps au-delà de `KESH_BANK_IMPORT_MAX_MB` (minimum 1 Mio) en `400
  VALIDATION_ERROR` sans `field`. *(Annoté le 2026-10-31, revue de code P1 de la 2b, A4 : depuis la remédiation de la revue de la 2a
  (C-v016-2-24), la limite des deux routes de saisie est propre, **3 Mio**, indépendante de `KESH_BANK_IMPORT_MAX_MB` ;
  500 lignes pleines passent, et l'asymétrie ci-dessus ne vaut plus qu'au-delà de 3 Mio.)* Le format
  se contrôle par `isValidAmount` **généralisée** (validation P1, R3/F4) : options `maxDecimals` (défaut 4, deux ici),
  `maxIntegerDigits` (défaut 15, treize ici), `allowNegative` (défaut non ; oui pour les soldes, non pour les montants
  de ligne) et `allowEmpty` (défaut oui, comme le journal ; non pour les soldes) — **aucune seconde expression
  régulière** ; la virgule est admise à la saisie et **normalisée en point** dans le corps JSON — par `parseAmount`
  puis `toFixed(2)` (un `Big` rend `"-120.5"` par `toString()`). Les longueurs se comptent en points de code
  (`[...s].length`, comme `chars().count()` côté serveur), **après `trim`**, des deux côtés. *Asymétrie assumée*
  (validation P2, R15) : le client compte les chiffres **saisis**, le serveur la valeur **normalisée** — `"50.000"` est
  refusé à l'écran et admis par l'API ; aucun test ne les exige identiques.
  Vitest : solde `-120,50` accepté et envoyé `"-120.50"` ; quatorze chiffres refusés ; solde vide refusé et sans écart
  affiché ; `"0"` refusé comme montant de ligne ; `balance.test.ts` inchangé et vert (défauts du journal intacts).
- **AC-22** — *Vérifier* appelle l'aperçu ; ses avertissements s'affichent (lignes en doublon désignées par leur rang,
  relevé identique, solde de départ attendu et saisi, relevés qui chevauchent), chacun avec sa case de confirmation
  quand la création l'exige. Toute modification **d'un champ du relevé** après l'aperçu le périme (cases décochées,
  *Enregistrer* inactif jusqu'au nouvel aperçu) ; cocher ou décocher une case ne le périme pas. Les cases cochées
  sont **envoyées** dans le corps (`confirmDuplicateFile`, `confirmDuplicateLines`, `confirmOpeningBalanceGap`) —
  Vitest sur le corps envoyé. Une erreur du serveur (`BANK_STATEMENT_*`) s'affiche traduite, la ligne fautive
  désignée. *(Validation P3, R5 :)* la réponse de l'aperçu est liée à l'**instantané** du corps envoyé — une réponse qui
  arrive après une modification est ignorée (patron `loadSeq` de `[id]/+page.svelte`) — ; la table de correspondance des
  rangs est celle de cet instantané ; le formulaire est figé pendant un envoi, et *Enregistrer* porte la garde
  `submitting` (patron `DeleteBankImportAction.svelte`). *(Validation P4, P4 :)* un `400 VALIDATION_ERROR` sans `field`
  (corps rejeté par l'extracteur de la 2a — en pratique, trop volumineux) s'affiche par une clé **générique** de l'écran
  (`bank-import-manual-*`, « relevé refusé : trop volumineux ou illisible »), ne désigne aucune ligne, ne montre pas le
  message français du serveur et ne rend pas l'aperçu « réussi » ; Vitest.
- **AC-23** — Succès : notification, retour à `/bank-import`, le relevé en tête de la liste. Dans la liste
  (`BankImportList.svelte`) et au détail (`/bank-import/[id]`), un import `MANUAL` s'affiche **« Saisie manuelle »** (et
  son numéro s'il en a un) au lieu du `filename`, et son format **« Saisie manuelle »** au lieu de `MANUAL` ; la boîte de
  suppression et sa notification parlent d'un « relevé saisi à la main » (clés propres, non le texte « Le fichier
  { $filename } »). **Inventaire fermé** (validation P1, F9) — pour un import `MANUAL`, des variantes propres de :
  `DeleteBankImportAction.svelte` — titre (`bank-import-delete-dialog-title`), bouton (`bank-import-delete-button`),
  paragraphe bloqué (`bank-import-delete-blocked`), corps (`bank-import-delete-dialog-body`, dont « réimporter le même
  fichier », faux pour une saisie), erreurs (`bank-import-delete-error-reconciled`, `-not-found`) — ; notification
  `bank-import-delete-success` (`[id]/+page.svelte:37`) ; au détail (`[id]/+page.svelte:113-118`), les libellés
  « Fichier », « Importé le », « Format source », le titre de page : passés en clés aux quatre catalogues —
  *(validation P3, R4, DRY)* en **réemployant** les clés existantes (`bank-import-labels-period`,
  `-opening-balance`, `-closing-balance`, `-transaction-count`, `-list-col-transactions`, `common-loading`), et en ne
  créant en `bank-import-detail-*` que les libellés neufs — avec, pour un relevé saisi, « Relevé », « Saisi le », « Saisie manuelle » et le numéro. **Complété à la
  validation P2 (F9)** : les en-têtes de la liste (`BankImportList.svelte:24-25`, `bank-import-labels-list-col-imported-at`
  « Date import », `…-col-filename` « Fichier ») prennent une forme qui vaut pour les deux origines ; **tout** le texte
  en dur du détail passe en clés — « Chargement… », « Période », « Solde ouverture », « Solde clôture », « Nombre de
  transactions », « Transactions », les en-têtes du tableau, le `<h1>` « Détail import bancaire » **et** le `<h2>`
  « Import #{id} », **et** (validation P3, R10 = F5) le `<title>` (`:88`), le lien « ← Retour aux imports » (`:92`) et
  les trois messages `'Identifiant invalide'` (`:62`), `'Import introuvable.'` (`:74`), `'Erreur inattendue.'` (`:76`) ;
  et les textes d'interface qui disent « importée » (validation P3, F4 ; **frontière récrite à la validation P4, P1**) :
  à la **2b**, `bank-accounts-close-dialog-body` (`fr-CH:1473`, `de-CH`/`it-CH`/`en-CH:1409`) et
  `fiscal-years-checks-k2-help` (`fr-CH:997`, `de-CH`/`it-CH`/`en-CH:946`) avec son repli
  `frontend/src/lib/features/fiscal-years/closing-checks-labels.ts:48` — pendant d'écran de la phrase K2 du manuel
  (`user-manual.tex:802`) —, **et** (validation P5, P1) le repli de `bank-accounts-close-dialog-body`,
  `frontend/src/routes/(app)/bank-accounts/+page.svelte:838-839` (identique aujourd'hui à `fr-CH:1473`), se récrivent
  pour valoir aussi d'un relevé saisi, comme le manuel (AC-27), **ensemble, au même commit que le catalogue** ; la
  valeur `fr-CH` récrite de `bank-accounts-close-dialog-body` **garde le verbe « rouvrir »** — la clé est une exemption de
  G8 et G8-bis (`crates/kesh-i18n/src/loader.rs:1193`, `:1308`), qui exigent qu'elle prescrive encore une réouverture
  (validation P6, P3). Mécanique des gardes, lue dans
  `frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts` : son premier test ne retient que les clés présentes au
  catalogue `fr-CH` qui portent **au moins deux** replis distincts (`parTexte.size > 1`), hors `TOLEREES` (l'allowlist,
  dont chaque entrée doit rester divergente) ; une clé à **un seul** site d'appel, comme ces deux-là, ne le fait donc pas
  rougir si seul le catalogue change — l'écran afficherait l'ancien repli tant que le catalogue n'est pas chargé. C'est
  ce que garde G13 (`REPLIS_A_SITE_UNIQUE`, même fichier : repli égal à la valeur `fr-CH`) ; les deux clés récrites
  ici y sont **ajoutées** — leurs sites d'appel sont lus par `findCallSites` : des `i18nMsg('<clé>', '<repli>')`
  littéraux dans des `.svelte`/`.ts` hors `.test.` (`i18n-harvest.js:32`, `i18n-literal-reader.js:491` ; écrit au
  présent à la validation P6, P4) —, et le décompte du commentaire de G13 (`:210`, « treize clés ») est **recompté** sur
  la liste finale. Avant de clore la liste, greper la valeur de repli dans `frontend/src` ; les deux
  **messages serveur** `bank-accounts-errors-has-transactions` et `bank-accounts-errors-has-pending-transactions`, dont
  le repli vit dans `errors.rs` (`:2714-2721`, `:2776-2780`), sont à la **2a** (son AC-11 : repli, quatre catalogues,
  `bank_accounts_e2e.rs:2485`) ; la 2b n'en garde que le Vitest qui recopie le texte
  (`frontend/src/routes/(app)/bank-accounts/bank-accounts-page.test.ts:502`) — *précisé à la validation P5, P3 :* ce
  Vitest recopie le texte de `has-pending-transactions` dans une valeur de mock (`mockRejectedValue`) et ne dépend pas
  du catalogue ; il ne se met à jour que si la 2a récrit ce message, pour rester un exemple vrai. La formulation cible
  des deux messages serveur revient à la 2a (constat P5 transmis : seul `has-pending-transactions` dit « importées »,
  `fr-CH:1439` ; `has-transactions`, `:1433`, ne le dit pas). `homepage-bank-no-transactions`
  n'est employée nulle part (clé morte, `grep -rn "no-transactions" frontend/src crates/*/src` hors `.ftl` : rien) :
  elle n'est **pas** récrite (LOW non pris de la validation P4) ; *(validation P5, P7 :)* elle est **tracée** — issue à
  ouvrir, parente, liste C de la release, point 12. Le refus « relevé identique » (`BANK_IMPORT_DUPLICATE_FILE`) porte le message serveur « Ce fichier a
  déjà été importé » (`errors.rs:2354-2358`) : l'écran de saisie affiche sa propre clé `bank-import-manual-*` et ne
  montre pas ce message.
- **AC-24** — i18n de l'écran : toutes les clés nouvelles du frontend (`bank-import-manual-*`, `bank-import-detail-*`,
  libellés des `reason`) dans
  les **quatre** catalogues (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`), aucun texte en dur dans le
  balisage ; `npm run lint-i18n-ownership` vert (préfixe `bank-import-` admis pour `features/bank-import/`, aucune clé
  d'un autre domaine réutilisée — pas de `journal-entry-form-*`). Gardes qui rougiront et se recomptent depuis la
  source (validation P1, F8) : `frontend/src/lib/shared/i18n-keys.test.ts` (`ATTENDU.sitesTotal`, cardinalité exacte),
  `i18n-un-repli-par-cle.test.ts`, `i18n-repli-divergent-actif.test.ts` (*glose corrigée à la validation P6, P2 :* divergences **actives** entre deux
  replis ou plus **d'une même clé** déjà servie par le catalogue — *précisé à la validation P7, P3* ; l'égalité repli = valeur `fr-CH` n'y est tenue que par G13, pour les clés
  **nommées** de `REPLIS_A_SITE_UNIQUE`, et par les préfixes de `PREFIXES_REPLI_EGAL_AU_CATALOGUE`, `:279`) — d'où :
  chaque **clé statique neuve** de l'écran (`bank-import-manual-*` hors la famille `-reason-`, et `bank-import-detail-*`),
  presque toutes à site unique, **et** chaque **clé existante dont le texte est récrit** — ici, outre les deux de l'AC-23 (étape 0, validation P8, P3),
  `bank-import-labels-list-col-imported-at` et `bank-import-labels-list-col-filename` (`BankImportList.svelte:24-25`,
  `fr-CH:1215-1216`, `de-CH`/`it-CH`/`en-CH:1154-1155` — *ajouté à l'étape 0 du développement, validation P8, P1 :* les
  **quatre** catalogues —, repli et catalogue au même commit ; validation P7, P2 ; les autres `bank-import-labels-*` de l'AC-23
  sont réemployées sans changement de texte) — est **ajoutée nommément** à `REPLIS_A_SITE_UNIQUE` (aucun préfixe ajouté à
  `PREFIXES_REPLI_EGAL_AU_CATALOGUE`), la liste nommée recopiée au Dev Agent Record, et le décompte du commentaire de
  G13 (`:210`) recompté sur elle ; **sauf** une clé statique dont le site n'est pas un `i18nMsg('<clé>', '<repli
  littéral>')` (G13 la ferait rougir) : elle s'écrit au Dev Agent Record avec la garde qui la tient, ou, s'il n'y en a
  aucune, en issue demandée (validation P7, P5) ; **angle mort assumé** : la famille
  dynamique `bank-import-manual-reason-` n'a pas de site littéral (clé composée d'un gabarit) et échappe à G13 — elle est
  gardée par le test croisé (égalité de la table et de `ManualStatementReason::ALL`) et par le Vitest de couverture de la
  table,
  `i18n-libelle-en-dur.test.ts`, `MOTIFS_DYNAMIQUES` et `CARDINALITES` (famille `bank-import-manual-reason-`), et la
  parité des catalogues (`crates/kesh-i18n/src/loader.rs`, `parity_between_locales`). *(Les clés des messages serveur
  et du libellé d'audit sont posées par la 2a, AC-11 et AC-15.)* *(Validation P3, R3 :)* compteurs à recompter, non à
  ajuster : `i18n-keys.test.ts:773` (`sitesGabarit`, 11 → 12), `:960` (`SITES_GABARIT_ATTENDUS`, une ligne neuve),
  `i18n-libelle-en-dur.test.ts:250` (`CANDIDATES_ATTENDUES`, 71 → à recompter si la table expose une fonction
  `…Label`), et `i18n-entrees-a-variables.test.ts` (clés à variables). **Test croisé des `reason`** (décision de
  l'orchestrateur à la validation P3, R1) : la table TypeScript des `reason` est
  lue par un **test Rust** — *forme imposée à la validation P4, P2 :* fichier
  `frontend/src/lib/features/bank-import/manual-statement-reason-label.ts`, **un seul** objet
  `export const MANUAL_STATEMENT_REASON_LABELS = {` … `};`, une entrée par ligne, clé littérale en début de ligne
  (`^\s*'?([a-z_]+)'?\s*:`), sans commentaire dans le bloc — (`crates/kesh-api/tests/`, patron `lire` de
  `textes_coherents.rs:46-48`), qui extrait ses clés et les égale, dans les deux sens, à `ManualStatementReason::ALL`
  mappé par `code()` (2a, AC-4) ; le test panique si le bloc est introuvable ou vide *(phrase remise en ordre à la
  validation P5, P6)*. **Forme de la valeur** (P6) : le tuple `[suffixe, repli]` du patron `CSV_LINE_ERROR_LABELS`
  (suffixe en tirets, famille `bank-import-manual-reason-` de `MOTIFS_DYNAMIQUES`) ; une entrée par ligne, **repli
  compris** — aucune ligne de continuation qui commencerait par un mot suivi de `:`. **Garantie de la 2a** — *rattachement de la
  P5 remplacé à la validation P6 (P1)* : le test croisé suppose que `ALL` est lié à l'énumération par une forme non
  circulaire ; ce que la 2b en tient pour acquis est **seulement ce qui est versionné**, vérifié au point bloquant de la
  liste « À recontrôler à l'étape 0 » ci-dessous ; le Vitest de
  couverture de la table reste, mais ne fait plus foi seul.
- **AC-25** — Recette visuelle (`CLAUDE.md` § *Recette visuelle*, thème clair) : captures, sur une base de
  démonstration, de `/bank-import` (le lien), du formulaire vide, d'un relevé de trois lignes **déséquilibré** (écart
  affiché, *Enregistrer* inactif), du même **équilibré** après aperçu, des avertissements (doublon, solde de départ),
  de la liste avec un relevé saisi, du détail, et de la boîte de suppression ; et aussi (validation P1, F10) : une ligne
  invalide avec la raison à côté du bouton, une erreur serveur avec la ligne désignée, l'aperçu périmé, l'avertissement
  « relevé identique » et l'information de chevauchement, les cases cochées, la notification de succès, le détail d'un
  relevé saisi avec ses transactions, `/bank-import` vu par un Consultation (lien absent), et le filtre d'action du
  journal d'audit avec l'option nouvelle ; et (validation P3, F10) soldes vides (aucun écart affiché), un relevé saisi
  **rapproché** (suppression inactive, paragraphe bloqué), les erreurs de suppression, la notification de
  **suppression**, et le refus d'un compte clos à la saisie. L'énumération est un minimum. Rangées dans
  `/home/gcorbaz/devel/kesh-gate-logs/v0162/captures/`, listées au Dev Agent Record et jointes au prompt de revue.
- **AC-26** — `frontend/tests/e2e/bank-manual-statement.spec.ts` *(récrit en partie à la validation P3, R7 : la base
  est **tronquée** par `seedTestState('with-company')` en `beforeAll`, `helpers/test-state.ts`, si bien que l'unicité des
  montants n'est pas requise ; l'aide `createAndValidateInvoiceViaApi` rend un TTC fixe, 972.90)* : en `beforeAll`,
  `seedTestState('with-company')` ; un Comptable (créé par une aide `createUser` **extraite** d'`audit-log.spec.ts:52`
  vers `helpers/`) crée et valide une facture par l'API — datée du jour (date **locale** de Zurich, non UTC : le piège
  que commente `createJournalEntryViaApi`), la ligne rapprochée datée du jour, les deux autres de la veille, sauf un
  1ᵉʳ janvier (alors du jour aussi), dans l'exercice ouvert du seed et dans
  la fenêtre de proposition (au plus 30 jours, Story 28-7, `reconciliation-amount-due.spec.ts:80-81`) — ; sur un
  compte bancaire créé par `wiredBankAccountViaApi` avec un IBAN suisse **synthétique fixe** (valide, comme les autres
  specs) — *récrit à la validation P4, P8 :* la base est tronquée en `beforeAll` et le preset `with-company` ne crée aucun
  compte bancaire, si bien qu'aucun relevé antérieur n'existe et qu'aucun avertissement de solde de départ ne peut
  surgir ; le paramètre `isPrimary` et l'IBAN « unique » de la P2 n'ont plus de motif et sont **retirés** ; ouvre `/bank-import`, suit le lien, saisit un relevé de **trois** lignes dont un crédit égal au TTC de la
  facture, constate l'écart, corrige le solde d'arrivée, *Vérifier*, *Enregistrer* ; retrouve le relevé « Saisie
  manuelle » dans la liste ; ouvre `/reconciliation`, choisit le compte, accepte la proposition de la facture ; ouvre
  `/audit-log`, filtre sur l'action « Relevé bancaire saisi à la main » **et** sur le type et l'identifiant de l'entité
  (l'import créé), et retrouve l'entrée. L'IBAN est validé à la création du compte par
  `kesh_core::types::Iban::new` (`routes/bank_accounts.rs:352`, `kesh-core/src/types/iban.rs`, ISO 13616, table des
  longueurs par pays) — *citation corrigée à la validation P4, P7 : `kesh-qrbill/src/validation.rs:40` est celle de la
  QR-facture*. Un second test : un
  utilisateur Consultation ne voit pas le lien.
- **AC-27** — Manuel utilisateur FR (`docs/manual/fr/user-manual.tex`, PDF régénéré) : une sous-section
  *Saisir un relevé à la main* (`\label{sec:releve-manuel}`) dans *Import bancaire* — quand l'employer (un compte sans
  relevé électronique), ce qu'on saisit, le contrôle d'équilibre exact, le solde de départ attendu, les doublons (pas
  d'« ignorer »), la correction (supprimer puis ressaisir ; annuler d'abord un rapprochement), et **la conduite si la
  banque se met plus tard au CAMT** (C-v016-2-6). Les phrases devenues fausses se récrivent : `user-manual.tex:2039-2041`
  (« Kesh supporte deux formats. Seul le CAMT.053 porte le solde de clôture […] »), `:255-258` (« Seuls les relevés
  CAMT.053 portent un solde de clôture »), `:2264-2277` (doublons : la saisie ne propose pas « ignorer » ; la phrase est à `:2277`, validation P3, F1). **Inventaire
  complété à la validation P1 (F1)** — *la citation « `:803` » de la première rédaction était fausse : la phrase K3
  est à `:813`* : `:813` (« Les relevés de fin d'exercice ne se comparent que pour les relevés CAMT, qui portent un
  solde ; un fichier CSV n'en porte pas » — et « un compte alimenté par CSV seulement » : ni saisi), `:814` (« une
  transaction importée » → importée ou saisie), `:2046-2052` (« À défaut de CAMT.053, importez le relevé en CSV… le
  format MT940 n'est pas lu » : **renvoyer** à `\ref{sec:releve-manuel}`, c'est le cas de #816), `:2126` (« portant un
  solde de clôture (un CAMT.053) »), `:2216` (« Un import CSV ne porte donc toujours pas de solde » : à relire),
  `:49` (« d'importer vos relevés bancaires (CAMT.053 et CSV) »), `:1925`, `:1958`, `:2029` (« transactions
  importées » : une transaction saisie compte de même), `:2097-…` (§ *Supprimer un import* : « le nom du fichier »,
  « le même fichier peut ensuite être réimporté ») et `:2462` (« puis le nom du fichier »), et, ajoutés à la validation
  P2 (F5) : `:232` (avec `:255-258`, la tuile : « si un relevé a été importé »), `:802` (K2 — *et non K3, validation P3, F3* : « Des transactions
  importées »), `:821` (« une transaction importée entre-temps »), `:2095` (« le nom du fichier ouvre le détail de
  l'import »), `:2292` (« passer en comptabilité les transactions bancaires importées »), puis, à la validation P3 (F2) : `:1971-1973`
  (liste des refus sur un compte clos : la saisie s'y ajoute), `:2012-2013` et `:2105-2106` (« Supprimer l'import », « en
  nommant le fichier »), `:2094` (« date, fichier, période… »), `:2120-2121` (trace « Import bancaire supprimé … avec le
  fichier »), `:2023-2024` (« réimporter » après réouverture) — l'inventaire nommé est un **minimum**, le filet aplati
  tranche ; et
  `admin-manual.tex:82` (« Import bancaire CAMT.053 … et CSV »). *(Le rapport F citait aussi `:2363` : c'est le rejet
  d'une transaction, qui ne nomme pas le fichier — écarté après lecture.)* Le filet n'est plus un `grep` de phrases
  coupées par les retours à la ligne : le texte **aplati** (`tr '\n' ' ' | tr -s ' '`) du `.tex` et du PDF est cherché
  sur `CAMT`, `CSV`, `import`, `fichier`, `solde de clôture`, et chaque occurrence est triée au Dev Agent Record. La
  conduite « la banque passe au CAMT » *(récrite à la validation P2, R4 = F1 : les trois cas de la P1 étaient faux pour
  le CAMT)* suit la **clé réelle** d'une ligne (`kesh-core/src/bank_imports.rs:365-377`) : date, montant, et le
  **premier identifiant non vide** parmi référence, identifiant de bout en bout et identifiant de transaction (casse et
  blancs ignorés). Une ligne saisie ne porte que la référence qu'on y tape ; une ligne d'un fichier CAMT porte presque
  toujours un identifiant — sa référence de créancier ou `NtryRef`, à défaut son `EndToEndId`, à défaut son
  `AcctSvcrRef` (`kesh-import/src/camt053/mod.rs:405` `NtryRef`, `:426-428` `EndToEndId`, `:412` `AcctSvcrRef`,
  `:629-630` ; citations complétées à la validation P3, F8). Le manuel dit donc : pour un fichier **CAMT**, la
  reconnaissance d'une ligne déjà saisie **n'est pas à attendre** — une ligne saisie sans référence y entre presque
  toujours **deux fois** ; elle n'est reconnue (et alors ignorée par défaut à l'import) que si la référence saisie égale
  le premier identifiant du fichier. Pour un CSV sans colonne d'identifiant, une ligne saisie sans référence a la même
  clé et est reconnue. *(Récrit à la validation P3, F6 :)* la voie normale : importer **à partir du lendemain** de la
  dernière période saisie ; à défaut, supprimer le relevé saisi (non rapproché, ou après annulation de chacun de ses
  rapprochements, `\ref{sec:annuler-rapprochement}`) avant d'importer la même période. La FAQ gagne une entrée
  « Ma banque ne propose ni CAMT.053 ni CSV » qui renvoie à la sous-section. La sous-section dit aussi ses limites
  (validation P3, F12) : CHF seule (Q8), 500 lignes au plus, refus sur un compte clos, et un solde de départ comparé au
  seul relevé **précédent à solde** (un import CSV n'en porte pas). Au PDF aplati, l'apostrophe est typographique :
  greper `’` ou un motif sans apostrophe (validation P3, F11).
  Les manuels DE/IT/EN ne sont pas traduits dans cette version : rien à y faire.
- **AC-29-bis** *(point 2 de l'AC-29 d'origine, déplacé au découpage)* — CHANGELOG `[0.17.0]` : l'entrée gagne une section
  **`### Ajouté`** (en tête des sections) : la saisie d'un relevé à la main, en une entrée, `refs` [#816] par un lien,
  sans mot-clé de fermeture ; les sections *Modifié* et *Corrigé* (#811, #812) restent. L'en-tête de l'entrée,
  récrit par la 2a (AC-29 points 3, 4 et 6), se relit pour qu'il annonce la fonctionnalité. *(Validation P3, F9 :)* la
  2b **complète** la phrase d'ouverture posée par la 2a (migration, sauvegarde, correctif) en y ajoutant la saisie
  manuelle ; l'entrée *Ajouté* dit en une demi-phrase le `sourceFormat` nouveau (`MANUAL`) et l'action d'audit « Relevé
  bancaire saisi à la main », pour les intégrateurs.
- **AC-30** — README, section *Fonctionnalités*, ligne *Import bancaire* (`README.md:41`) : « **saisie manuelle d'un
  relevé** quand la banque n'en fournit pas de fichier ✓ ». *(La ligne de version de la feuille de route est posée par
  la 2a, AC-29 point 8.)*

## Tâches

1. **Étape 0** — recontrôler au code chaque ligne du § *À recontrôler* ci-dessous, et ce que la 2a a livré (routes,
   codes, `reason`, action d'audit), ligne lue citée au Change Log.
2. **Tests rouges d'abord** : le Vitest de `manual-statement.ts` (AC-21) et de la correspondance des rangs (AC-20).
3. `frontend` : types, `manual-statement.ts`, options d'`isValidAmount`, `ManualStatementForm.svelte`, page, lien,
   libellés de liste, détail et suppression, table des `reason`, i18n (AC-19 à AC-24) ; Vitest.
4. E2E Playwright (AC-26) ; captures (AC-25).
5. Textes : manuel utilisateur (AC-27, PDF au même commit que le `.tex` : `make user` — et `make admin` pour
   `admin-manual.tex:82` — dans `docs/manual/`, sous `flock "$GATE_LOGS/.gate.lock"` et `scripts/mem-guard.sh`,
   **macros `\keshVersion`… inchangées** jusqu'au tag final, parente A.3 ; consignes reprises de la 2a, validation P3,
   F7), CHANGELOG *Ajouté* (AC-29-bis), README (AC-30) ;
   **greper la valeur** de chaque phrase corrigée (§ *Propagation* de la parente).
6. Gates, mutations, Dev Agent Record.

## Tests et mutations prévus

Fichiers prévus : `frontend/src/lib/features/bank-import/manual-statement.test.ts`, `ManualStatementForm.test.ts`, la
table et le test des `reason`, les tests existants de `BankImportList`, `DeleteBankImportAction`, de la page détail,
`journal-entries/balance.test.ts` (défauts inchangés), `frontend/src/lib/shared/i18n-keys.test.ts` ;
`frontend/tests/e2e/bank-manual-statement.spec.ts`, `frontend/tests/e2e/helpers/api-fixtures.ts` (`createUser`
extraite) ; `crates/kesh-api/tests/` (test croisé des `reason`, AC-24).

| # | Mutation (une à la fois, motif vérifié unique, restauration `git checkout -- f && touch f`) | Doit rougir |
|---|---|---|
| M13 | `manual-statement.ts` : les débits ajoutés au lieu d'être retranchés | Vitest AC-21 |
| M14 | *Enregistrer* actif quand l'écart est non nul | Vitest AC-21 |
| M15 | aperçu non périmé par une modification du formulaire | Vitest AC-22 |
| M19 | formulaire : les cases cochées non transmises dans le corps | Vitest AC-22 (corps envoyé) |
| M25 | rangs : le rang envoyé affiché tel quel, sans la table de correspondance | Vitest AC-20 (ligne vide au milieu) |
| M26 | normalisation : `toFixed(2)` retiré | Vitest AC-21 (`"-120.50"` attendu) |
| M27 | `isValidAmount` : défaut `allowEmpty` inversé | `balance.test.ts` (défaut du journal) |
| M28 | lien de saisie affiché à tout rôle | Vitest de la page / E2E AC-26 (Consultation) |
| M29 | réponse d'aperçu tardive acceptée (instantané ignoré) | Vitest AC-22 |
| M30 | test croisé des `reason` : une clé retirée de la table TypeScript | test Rust croisé (AC-24) |

Chaque mutation : le test qui rougit et la raison, au Dev Agent Record. Une mutation **survivante** est un défaut de test
à corriger, non à déclarer.

## Gates

Au **dernier commit de code** de la 2b : gate backend complet (`scripts/test-fast.sh --ci` sous verrou — les catalogues
de `kesh-i18n` sont lus par le backend), Vitest complet, `npm run check`, `npm run lint-i18n-ownership`,
`npm run build`, et la **suite E2E complète** selon `docs/testing.md` à la lettre (dossiers propres au worktree passés au
backend **et** au runner) ; échecs jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus*. *(Repris
de la 2a à la validation P3, F7 :)* base de gate **remise à zéro** avant chaque gate complet ; démarrage réel contre la
base persistante `kesh_e2e_v0162` (P8, la migration de la 2a) ; `make` des manuels sous verrou et `mem-guard`, macros
inchangées. Ne déclarer que ce qui a tourné, chiffres recomptés.

## Fichiers touchés (prévus)

| Module | Fichiers |
|---|---|
| `frontend` | `src/lib/features/bank-import/{ManualStatementForm.svelte, manual-statement.ts, *.test.ts, bank-import.api.ts, bank-import.types.ts, BankImportList.svelte, DeleteBankImportAction.svelte}`, la table des `reason`, `src/lib/features/journal-entries/balance.ts`, `src/routes/(app)/bank-import/+page.svelte`, `src/routes/(app)/bank-import/manual/+page.svelte` (+ `+page.ts`, garde), `src/routes/(app)/bank-import/[id]/+page.svelte`, `src/lib/shared/i18n-keys.test.ts`, `src/lib/shared/i18n-libelle-en-dur.test.ts`, `tests/e2e/bank-manual-statement.spec.ts`, `tests/e2e/helpers/api-fixtures.ts` (l'aide `createUser` extraite), `tests/e2e/audit-log.spec.ts` (`createUser` extraite) |
| `kesh-api` (tests) | le test croisé des `reason` (AC-24) |
| `kesh-i18n` | `locales/{fr,de,it,en}-CH/messages.ftl` (clés de l'écran ; `bank-accounts-close-dialog-body` et `fiscal-years-checks-k2-help` récrites — numéros par catalogue à l'AC-23 ; `bank-import-labels-list-col-imported-at` et `bank-import-labels-list-col-filename` récrites, `fr-CH:1215-1216`, `de-CH`/`it-CH`/`en-CH:1154-1155` — étape 0, validation P8, P1) |
| `frontend` (autres) | `src/lib/features/fiscal-years/closing-checks-labels.ts` (`:48`), `src/routes/(app)/bank-accounts/+page.svelte` (`:838-839`, repli), `src/routes/(app)/bank-accounts/bank-accounts-page.test.ts` (`:502`), `src/lib/shared/i18n-repli-divergent-actif.test.ts` (`REPLIS_A_SITE_UNIQUE`) |
| docs | `docs/manual/fr/user-manual.tex` (+ PDF), `docs/manual/fr/admin-manual.tex:82` (+ PDF), `CHANGELOG.md` (*Ajouté*), `README.md:41` |

## À recontrôler à l'étape 0 du développement

- ⛔ **Renvoyé de la validation P8 (clôture au plafond)** — rapport `/home/gcorbaz/devel/kesh-gate-logs/v0162b-validate-p8.md`,
  P1 (MEDIUM) et P2 à P4 (LOW), **à appliquer** et à déclarer au Change Log, ligne lue citée : les deux en-têtes
  `bank-import-labels-list-col-imported-at` et `bank-import-labels-list-col-filename` se récrivent dans les **quatre**
  catalogues (`fr-CH:1215-1216`, `de-CH`/`it-CH`/`en-CH:1154-1155`), au même commit que leur repli
  (`BankImportList.svelte:24-25`) ; la ligne `kesh-i18n` de la table « Fichiers touchés » et la liste « Les citations des
  AC » ci-dessous sont complétées de ces deux clés et de ces numéros ; le Vitest V13
  (`frontend/src/lib/features/bank-import/BankImportUpload.test.ts:735`, `FR_TEXTS`, et `:845`) est récrit avec les
  nouveaux textes ; à l'AC-24, « ici » se lit « ici, outre les deux clés de l'AC-23 » (quatre clés existantes récrites
  en tout) ; au point bloquant, « noms compris » ne fait pas arrêter sur un simple renommage — ce qui compte, ce sont
  les éléments (index sans joker, `COUNT`, `ALL`, les deux assertions, le test de comptage par `include_str!`).
- ⛔ **Point bloquant** (validation P6, P1 ; *récrit à la validation P7, P1, sur C-v016-2-20 tel que le registre le porte,
  `v016-2-choix-autonomes.md:434-450`*) : le développeur de la 2b vérifie, **dans le code commité de la 2a** (non dans la
  fiche), pour `ManualStatementReason` au moins (`crates/kesh-core/src/bank_imports/manual.rs` et ses tests) : `index()`
  par `match` exhaustif sans joker ; `pub const COUNT` à côté ; `pub const ALL` ; un test `ALL.len() == COUNT` et
  `ALL[i].index() == i` ; **et** la pièce maîtresse de C-v016-2-20, le test qui compte les variantes **dans le texte
  source** de l'énumération (`include_str!("manual.rs")`) et l'égale à `COUNT` et à `ALL.len()` — sans lui, une variante
  ajoutée avec son bras d'`index`, `ALL` et `COUNT` inchangés, laisse les deux premières assertions vertes (registre
  `:442-447`). Relevé au 2026-10-30 dans l'arbre de travail, **non encore commité** : `manual.rs:84` (`COUNT`), `:87`
  (`ALL`), `:102` (`index`), `manual_tests.rs:133` (`all_is_indexed_and_counted`), `:153` (`include_str!`), `:173`
  (`all_covers_every_declared_variant`) — à relire au commit, noms compris *(étape 0, validation P8, P4 : les noms relevés
  sont indicatifs ; ce qui manque s'entend d'un élément, non d'un nom)*. Si l'un manque, le développeur de la 2b
  **s'arrête et le signale** à l'orchestrateur, sans écrire le test croisé de l'AC-24.
- Que `GET /companies/current` ne rend que les comptes ni archivés ni clos (`docs/api-external.md:240`).
- `frontend/src/routes/(app)/supplier-invoices/import/+page.ts` — le patron de garde Comptable+ (`load()` redirige un rôle
  autre que Admin/Comptable vers `/`), à reprendre pour `/bank-import/manual` *(les deux puces jumelles fusionnées à la
  validation P3, R9 = F8)*.
- Les citations des AC (ajoutées à la validation P3, R9 = F8) : `[id]/+page.svelte:37`, `:62`, `:74`, `:76`, `:88`, `:92`,
  `:113-118` ; `BankImportList.svelte:24-25` ; `helpers/api-fixtures.ts:100-105` *(`:190`, ligne d'`isPrimary`, ôtée à l'étape 0 de la 2a — validation P5 de la 2a, P3)* ; `audit-log.spec.ts:52` ;
  `reconciliation-amount-due.spec.ts:80-81` ; `kesh-core/src/bank_imports.rs:365-377` ;
  `kesh-import/src/camt053/mod.rs:405`, `:412`, `:426-428`, `:629-630` ; `errors.rs:2354-2358` ;
  `i18n-keys.test.ts:773`, `:882`, `:960` ; `i18n-libelle-en-dur.test.ts:250` ; `messages.ftl` (`fr-CH:997`, `:1215-1216`, `:1473` ;
  `de-CH`/`it-CH`/`en-CH:946`, `:1154-1155`, `:1409`) ; `closing-checks-labels.ts:48` ; `bank-accounts-page.test.ts:502` ; `bank-accounts/+page.svelte:838-839` et
  `i18n-repli-divergent-actif.test.ts` (`TOLEREES`, `REPLIS_A_SITE_UNIQUE`, ajoutés à la validation P5) ;
  `user-manual.tex:1971-1973`, `:2277` ; et (validation P4, P7) `routes/bank_accounts.rs:352`, `kesh-core/src/types/iban.rs`,
  `textes_coherents.rs:46-48`, `helpers/test-state.ts` (base tronquée).

## Dev Agent Record

### Agent Model Used

Opus 5.5 (Claude Code, agent de développement, autonomie), 2026-10-30/31. Worktree `/home/gcorbaz/devel/kesh-0163`,
branche `story/v016-2b-releve-manuel-ecran` (partie de `285d2fde`, la v016-2a revue fusionnée en `c87d0be7`), bases
`kesh_v0162b` et `kesh_e2e_v0162b`, port 3163, cible `kesh-0163/target`. Journaux : `kesh-gate-logs/v0162b/`.

### Étape 0

Au Change Log (entrée « Développement, étape 0 ») : point bloquant C-v016-2-20 **présent** dans le code commité de la 2a
(lignes citées), renvois P8 (MEDIUM P1, LOW P2 à P4) appliqués, autres lignes de « À recontrôler » lues.

### Completion Notes List

**Rouges d'abord** (commit `b61da862`, `rouge-1.log`) : `manual-statement.test.ts`, `manual-statement-reason-label.test.ts`
et `ManualStatementForm.test.ts` commités avant le code — **3 fichiers rouges, 0 test exécuté** (« Failed to resolve import
"./manual-statement" »). Puis le code : les trois verts au premier passage (67 tests avec `balance.test.ts`).

**Ce qui est livré** : AC-19 (lien, garde `+page.ts` patron de `supplier-invoices/import`) ; AC-20 à AC-22
(`manual-statement.ts` pur, `ManualStatementForm.svelte`, `isValidAmount` généralisée par options sur **une** expression
paramétrée, défauts du journal inchangés) ; AC-23 (`bank-import-display.ts`, liste, détail entièrement en clés, variantes
`bank-import-manual-delete-*`, notification de suppression propre ; K2 et boîte de clôture récrites — « rouvrir » gardé ;
`bank-accounts-page.test.ts:502` suit le message récrit par la 2a) ; AC-24 (clés aux quatre catalogues, gardes recalées,
G24) ; AC-25 (captures) ; AC-26 (E2E) ; AC-27 (manuel utilisateur + PDF, `admin-manual.tex:82` + PDF) ; AC-29-bis
(CHANGELOG *Ajouté*, phrase d'ouverture complétée) ; AC-30 (README). Après la fusion de la 2a revue : le `409
RECONCILIATION_ACCOUNT_LOCKED` s'affiche par son message serveur (générique, traduit), sans ligne désignée, l'aperçu
gardé pour réessayer — deux tests ajoutés (commit `test(v016-2b) : le 409…`).

**Compteurs de garde recalés** (ancienne → nouvelle valeur, recomptés) : `i18n-keys.test.ts` — `sitesTotal` 2619 → 2711
(+92, `grep -oE '(i18nMsg|[^a-zA-Z]msg)\('` aux deux bornes `285d2fde` → tête, ventilation par fichier au commentaire),
`sitesGabarit` 11 → 12, `SITES_GABARIT_ATTENDUS` +1 ligne, `MOTIFS_DYNAMIQUES` 9 → 10 préfixes (+`bank-import-manual-reason-`,
11 valeurs), `CARDINALITES` +1 (lue depuis `MANUAL_STATEMENT_REASON_LABELS`) ; `i18n-libelle-en-dur.test.ts` —
`CANDIDATES_ATTENDUES` 71 → 76, `conforme` 57 → 62 (cinq déclarations nommées) ; `i18n-repli-divergent-actif.test.ts` —
`REPLIS_A_SITE_UNIQUE` 13 → 91 entrées (4 clés récrites + 74 clés statiques neuves : 60 `bank-import-manual-*` hors
`-reason-`, 14 `bank-import-detail-*`), décompte du commentaire « treize » → « quatre-vingt-onze » (`awk … | grep -cE` :
91). Liste nommée : celle du fichier (`:217-…`), égale à l'ensemble des clés `bank-import-(manual|detail)-*` hors
`-reason-`/`-errors-` du catalogue fr-CH (`diff` vide). **Aucune clé statique sans repli littéral.** Angle mort assumé :
la famille `bank-import-manual-reason-` (gabarit), tenue par `MOTIFS_DYNAMIQUES`/`CARDINALITES`, le Vitest de couverture
et G24. `textes_coherents.rs` : G24 ajoutée (deux tests).

**Le manuel** (`make user admin`, sous verrou, `make-manuels-1.log` : trois passes chacun, stables, 0 référence
indéfinie) ; PDF aplati (`grep -oF … | wc -l`) : `Saisir un relevé à la main` 4, `Si la banque passe un jour au CAMT` 2,
`Ma banque ne propose ni CAMT.053 ni CSV` 2, `à partir du lendemain` 1, `n’offre pas` 2, `Seul le CAMT` 0, `Seuls les
relevés` 0, `Kesh supporte deux formats` 0, `??` 0 ; admin : `saisie à la main d’un relevé` 1. Filet du `.tex` aplati
trié (`user-tex-plat.txt`) : les occurrences restantes de `deux formats` (« deux formats de fichier … et accepte aussi
un relevé saisi »), `CSV n'en a` (trois, justes), `le nom du fichier` (cinq, dont une hors sujet — l'import de factures —,
les autres complétées de « ou Saisie manuelle »), `réimport` (quatre, chacune de fichiers), `importé` (justes ou
complétées). `:2216` relu : juste (préambule CSV). Sites récrits : `:49`, `:232`, `:255-258`, `:802`, `:813`, `:814`,
`:821`, `:1925`, `:1958-1959`, `:1973`, `:2012`, `:2023-2024`, `:2029`, `:2039-2041`, `:2046-2052`, `:2094-2095`,
`:2097-…` (supprimer), `:2120-2121`, `:2126`, `:2277`, `:2292`, `:2462`, `:2470`, plus la sous-section et la FAQ.

**Écarts à la fiche, et pourquoi** :
- ⚠️ Éditions hors de l'outil d'édition : `bank-import.types.ts` (ajout en fin par `cat >>`), `bank-import.api.ts`
  (remplacement à motif vérifié unique par `python3`), et l'ajout en fin de trois fichiers de tests
  (`DeleteBankImportAction.test.ts`, `bank-import-detail-page.test.ts`, `bank-import-page.test.ts`) et de
  `BankImportList.test.ts` (créé) par `cat`. Diffs relus ; tout le reste par Edit/Write.
- Mes choix, prévus C-v016-2-24 à 27, sont **renumérotés C-v016-2-25 à 28** à la fusion de la 2a revue (qui a pris
  C-v016-2-24), sur instruction de l'orchestrateur.
- Le test croisé des `reason` vit dans `textes_coherents.rs` (G24), non dans un fichier neuf (C-v016-2-27).
- Les captures sont prises par un troisième test E2E, sauté hors `KESH_E2E_CAPTURES_DIR` (C-v016-2-28).
- Deux gardes ont rougi au gate complet après fusion, sur mon code : l'inventaire des boutons rouges (le « × » de retrait
  portait `text-destructive`) et `e2e-selecteurs-traduits.test.ts` (deux `getByText` sur un libellé traduit dans la
  spec) — corrigés au commit `67ad0971`, puis gate et E2E rejoués.
- Un fichier vide a peut-être été créé hors du scratchpad : `/tmp/claude-1000/a.txt` (redirection d'un `sort` dans une
  commande de comparaison) — non supprimé (aucun `rm`), sans contenu de données.

### Gates (réellement exécutés, au dernier commit de code `67ad0971`, après fusion de `920ddb4d`)

Sous verrou (`gate-complet-3.log`) :
- **Mutations** (`muter.py`, journaux `mutations/M*.log`, bilan `mutations/bilan.txt`) : **10 jouées, 10 tuées, 0
  survivante** ; `git status` propre après la campagne (hors ce prompt non suivi).
- **Base** `kesh_v0162b` supprimée, recréée, migrée (87 + celles de la 2a revue : aucune nouvelle), seedée.
- **Backend** `scripts/test-fast.sh --ci` (`gate-1.log`) : garde des droits, `fmt`, `clippy -D warnings` verts ;
  nextest **4521 exécutés, 4521 passés, 7 ignorés**.
- **Frontend** : `npm run check` 0 erreur, 29 avertissements (préexistants, aucun sur un fichier de la story) ;
  `lint-i18n-ownership` PASS ; `npm run test:unit` **185 fichiers, 2200 tests, tous verts** (`vitest-1.log`) ; `npm run
  build` vert.
- **E2E complet** (`e2e-1.log`, recette `docs/testing.md`, port 3163, base persistante `kesh_e2e_v0162b` à 87 migrations,
  `/health` `version 0.17.0`, dossiers neufs `/tmp/kesh-e2e-v0162b-2/` passés au backend et au runner) : **327 passés,
  7 échoués, 18 ignorés** (13,4 min). Les sept échecs, jugés fichier par fichier contre § *Les échecs attendus* :
  `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` — les
  sept de la **KF-029 (#97)** ; aucun hors liste. Le parcours neuf `bank-manual-statement.spec.ts` : 2 passés, le test de
  captures ignoré (attendu). Backend arrêté par son PID.
- **Captures** rejouées après le dernier commit (`e2e-spec-2.log`, 3 passés).

### Mutations

| # | Mutation jouée | Test(s) rouge(s) |
|---|---|---|
| M13 | débits ajoutés (`.plus(totalDebits)`) | `computeManualTotals` ×2, `saveBlockReason` ×2 (`manual-statement.test.ts`) |
| M14 | raison « écart » seulement si l'écart est nul… absente (`gap === null`) | `M14 — un aperçu réussi…` (logique et formulaire) |
| M15 | `previewFresh` sans comparaison de clé | `M15 — une modification après l'aperçu le périme…`, et M29 |
| M19 | cases forcées à `false` dans le corps | `M19 — les cases cochées partent dans le corps…` |
| M25 | `screenRowOf` rend le rang tel quel | `M25 — le rang 1-based…`, les deux « CETTE ligne », `M25` du formulaire |
| M26 | `toFixed(2)` → `toString()` | `M26 — « -120,50 » part en « -120.50 »`, et M19 (`openingBalance` `1000.00`) |
| M27 | défaut `allowEmpty` inversé | `balance.test.ts` : `accepte chaîne vide` et trois `computeBalance` |
| M28 | lien affiché à tout rôle (`{#if true}`) | `Consultation ⇒ aucun lien (M28)`, `aucun utilisateur ⇒ aucun lien (M28)` |
| M29 | réponse d'aperçu liée à la clé courante | `M29 — une réponse d'aperçu arrivée après une modification…` |
| M30 | `unknown_direction` retirée de la table | `g24_la_table_des_reason_egale_l_ensemble_ferme_du_serveur` (Rust) |

### Captures (AC-25, thème clair, base E2E du worktree)

`/home/gcorbaz/devel/kesh-gate-logs/v0162b/captures/` : `01-bank-import-lien`, `02-formulaire-vide`,
`03-soldes-vides-sans-ecart`, `04-desequilibre-ecart`, `05-ligne-invalide-raison`, `06-erreur-serveur-ligne-designee`,
`07-equilibre-verifie`, `08-apercu-perime`, `09-notification-succes-et-liste`, `10-detail-releve-saisi`,
`11-boite-de-suppression`, `12-avertissements-identique-doublons-chevauchement`, `13-cases-cochees`,
`14-avertissement-solde-de-depart`, `15-releve-rapproche-suppression-bloquee`, `16-erreur-de-suppression`,
`17-notification-de-suppression`, `18-refus-compte-clos`, `19-journal-audit-filtre-action`, `20-consultation-sans-lien`.
Manque déclaré : l'erreur de suppression « rapprochée entre-temps » (409) n'a pas de capture propre (couverte par le
Vitest de `DeleteBankImportAction`).

### Défauts observés hors périmètre

Aucun nouveau. Rappels : issues de la liste C de la parente (période déjà saisie à l'import ; clé morte
`homepage-bank-no-transactions`) à ouvrir par l'orchestrateur.

### File List

- Nouveaux : `frontend/src/lib/features/bank-import/{ManualStatementForm.svelte, ManualStatementForm.test.ts,
  manual-statement.ts, manual-statement.test.ts, manual-statement-reason-label.ts, manual-statement-reason-label.test.ts,
  bank-import-display.ts, BankImportList.test.ts}`, `frontend/src/routes/(app)/bank-import/manual/{+page.svelte,+page.ts}`,
  `frontend/tests/e2e/bank-manual-statement.spec.ts`, `v016-2b-review-prompt-p1.md`.
- Modifiés : `frontend/src/lib/features/bank-import/{BankImportList.svelte, BankImportUpload.test.ts,
  DeleteBankImportAction.svelte, DeleteBankImportAction.test.ts, bank-import.api.ts, bank-import.types.ts}`,
  `frontend/src/lib/features/journal-entries/balance.ts`, `frontend/src/lib/features/fiscal-years/closing-checks-labels.ts`,
  `frontend/src/lib/shared/{i18n-keys,i18n-libelle-en-dur,i18n-repli-divergent-actif}.test.ts`,
  `frontend/src/routes/(app)/bank-accounts/{+page.svelte, bank-accounts-page.test.ts}`,
  `frontend/src/routes/(app)/bank-import/{+page.svelte, bank-import-page.test.ts, [id]/+page.svelte,
  [id]/bank-import-detail-page.test.ts}`, `frontend/tests/e2e/{audit-log.spec.ts, helpers/api-fixtures.ts}`,
  `crates/kesh-api/tests/textes_coherents.rs`, `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`,
  `docs/manual/fr/{user-manual,admin-manual}.{tex,pdf}`, `CHANGELOG.md`, `README.md`.
- Fiches : cette sous-fiche, `v016-2-choix-autonomes.md` (C-v016-2-25 à 28), `sprint-status.yaml`.

## Change Log

- 2026-10-30 — Sous-fiche créée au découpage de la Story v016-2 (validation P2, C-v016-2-18) ; AC repris de la fiche
  parente à leur état remédié de la P2. Prochaine étape : validation P3 (`v016-2b-validate-prompt-p3.md`).
- 2026-10-30 — **Validation P3** (Sonnet, lentilles R et F, contexte frais ; rapports
  `kesh-gate-logs/v0162b-validate-p3-{R,F}.md`) : **0 CRITICAL, 0 HIGH, 1 MEDIUM, 21 LOW** (R : 1 MEDIUM, 9 LOW ; F :
  0 MEDIUM, 12 LOW). Prémisses vérifiées au code (`csv-line-error-label.test.ts`, `i18n-keys.test.ts:773`,
  `i18n-libelle-en-dur.test.ts:250`, `audit-log.spec.ts:52`, `camt053/mod.rs:405`, `:426-428`, `user-manual.tex:1971-1973`,
  `:2277`, `closing_checks.rs:10-11`, `messages.ftl:166`, `:1433`, `:1439`, `:1473`, `textes_coherents.rs:46-48`).
  Traitement :
  - **MEDIUM** — R1 : décision de l'orchestrateur — ensemble fermé des `reason` épinglé côté Rust par la 2a
    (`ManualStatementReason::ALL`), test Rust croisé qui lit la table TypeScript posé par la 2b (AC-24, M30).
  - **LOW appliqués** — R2 (causes et ordre de la raison, *Vérifier*, M14), R3 (compteurs i18n), R4 (clés réemployées,
    DRY), R5 (instantané de l'aperçu, `submitting`), R6 (date de valeur, `trim`, ligne vide, désignation du champ), R7
    (AC-26 : base tronquée, TTC fixe, `createUser` extraite, IBAN MOD-97, dates locales, filtre par entité), R8 (M25 à
    M29), R9 = F8 (liste « À recontrôler »), R10 = F5 (`<title>`, retour, messages), F1 (`:2277`), F2 (sites du manuel),
    F3 (K2), F4 (textes d'interface « importée »), F6 (conduite CAMT), F7 (consignes de gate et de PDF reprises de la 2a),
    F9 (phrase d'ouverture et entrée *Ajouté*), F10 (captures), F11 (apostrophe typographique), F12 (limites au manuel).
  - **LOW non pris** : aucun.
  Prochaine étape : validation P4 ciblée (`v016-2b-validate-prompt-p4.md`, Opus).
- 2026-10-30 — **Validation P4** (ciblée, Opus ; rapport `kesh-gate-logs/v0162b-validate-p4.md`) : **0 CRITICAL, 0 HIGH,
  1 MEDIUM, 9 LOW**. Prémisses vérifiées au code (`errors.rs:2715`, `:2777` ; `bank_accounts_e2e.rs:2485` ;
  `bank-accounts-page.test.ts:502` ; `closing-checks-labels.ts:48` ; numéros des quatre catalogues ;
  `homepage-bank-no-transactions` sans usage ; `routes/bank_accounts.rs:352`). Traitement :
  - **MEDIUM** — P1 : décision de l'orchestrateur — les deux messages serveur `bank-accounts-errors-has-*` passent à la
    2a (repli `errors.rs`, `bank_accounts_e2e.rs:2485`), la 2b garde le Vitest `bank-accounts-page.test.ts:502` ;
    `fiscal-years-checks-k2-help` et son repli ajoutés ; numéros par catalogue.
  - **LOW appliqués** — P2 (forme de la table TypeScript, `ALL` public : 2a), P3 (exhaustivité de `ALL` : 2a, M23-bis),
    P4 (`400 VALIDATION_ERROR` sans `field` : message générique, asymétrie de taille écrite), P5 (`field` d'entête), P6
    (séquence de la raison), P7 (citation de l'IBAN, « À recontrôler »), P8 (`isPrimary` et IBAN unique retirés, motif
    caduc), P9 (parente : 2a), P10 (registre C-v016-2-19 ; borne -19).
  - **LOW non pris** : `homepage-bank-no-transactions`, clé morte, non récrite et sans issue (décision de l'orchestrateur).
  Prochaine étape : validation P5 ciblée (`v016-2b-validate-prompt-p5.md`, Sonnet).
- 2026-10-30 — **Validation P5** (ciblée, Sonnet ; rapport `kesh-gate-logs/v0162b-validate-p5.md`) : **0 CRITICAL, 0 HIGH,
  2 MEDIUM, 5 LOW**. Prémisses vérifiées au code (`bank-accounts/+page.svelte:837-840` ; `i18n-repli-divergent-actif.test.ts`
  : en-tête, `TOLEREES` `:51`, premier test `:147-151`, G13 `REPLIS_A_SITE_UNIQUE`). *Prémisse de P1 corrigée en
  partie* : récrire le catalogue seul ne fait **pas** rougir la garde des divergences actives (elle ne retient que les clés
  à deux replis distincts et plus) — la clé à site unique relève de G13 ; le remède demandé tient. Traitement :
  - **MEDIUM** — P1 : repli `bank-accounts/+page.svelte:838-839` à l'AC-23, aux fichiers touchés et à « À recontrôler »,
    récrit au même commit que le catalogue, clé ajoutée à `REPLIS_A_SITE_UNIQUE` ; P2 : décision de l'orchestrateur — la
    forme non circulaire est prescrite au développeur de la 2a (C-v016-2-20) ; la 2b écrit seulement que son test croisé
    s'appuie sur cette garantie (AC-24).
  - **LOW appliqués** — P3 (côté 2b : le Vitest `:502` n'est qu'un mock ; la formulation des messages serveur revient à la
    2a, constat transmis), P4 (registre `:370` annoté), P5 (borne de la parente à -19), P6 (phrase de l'AC-24 remise en
    ordre, forme de la valeur), P7 (`homepage-bank-no-transactions` tracée : parente, liste C, point 12 — l'entrée P4 qui la
    disait « sans issue » n'est pas réécrite).
  - **LOW non pris** : aucun.
  Prochaine étape : validation P6 ciblée (`v016-2b-validate-prompt-p6.md`, Opus).
- 2026-10-30 — **Validation P6** (ciblée, Opus ; rapport `kesh-gate-logs/v0162b-validate-p6.md`) : **0 CRITICAL, 0 HIGH,
  2 MEDIUM, 4 LOW**. Prémisses vérifiées au code (`i18n-repli-divergent-actif.test.ts:142`, `:210`, `:279` ;
  `crates/kesh-i18n/src/loader.rs:1193`, `:1308` ; la 2a et le registre au commit `a458b560` : aucune C-v016-2-20).
  Remédiation minimale (sixième passe, plafond à huit). Traitement :
  - **MEDIUM** — P1 : décision de l'orchestrateur — point bloquant en tête de « À recontrôler » (C-v016-2-20 au registre,
    AC-4 et M23-bis de la 2a sous la forme `index()`, `COUNT`, `ALL[i].index() == i`, sinon arrêt et signalement) ; le
    rattachement de l'AC-24 renvoie à ce point. P2 : glose de la garde corrigée ; clés statiques neuves ajoutées
    nommément à `REPLIS_A_SITE_UNIQUE`, sans préfixe ; angle mort de la famille `bank-import-manual-reason-` écrit.
  - **LOW appliqués** — P3 (« rouvrir » gardé, G8/G8-bis), P4 (« treize » recompté, condition au présent), P5 (phrase de
    l'AC-23 remise en ordre, « ces deux-là »), P6 (parente, liste C point 10 au pluriel).
  - **LOW non pris** : aucun.
  Prochaine étape : validation P7 ciblée (`v016-2b-validate-prompt-p7.md`, Sonnet).
- 2026-10-30 — **Validation P7** (ciblée, Sonnet ; rapport `kesh-gate-logs/v0162b-validate-p7.md`) : **0 CRITICAL, 0 HIGH,
  2 MEDIUM, 3 LOW**. Lignes lues : registre `v016-2-choix-autonomes.md:434-450` (C-v016-2-20, commité en `89415374`) ;
  `crates/kesh-core/src/bank_imports/manual.rs:84`, `:87`, `:102` et `manual_tests.rs:133`, `:153`, `:173` (arbre de
  travail de la 2a, non commité au moment de la remédiation) ; `BankImportList.svelte:24-25` ; `grep -n "bank-import-labels-"`
  sur la 2b (`:97`, `:100`) ; `i18n-repli-divergent-actif.test.ts:142`, `:210`. Remédiation minimale (avant-dernière passe).
  Traitement :
  - **MEDIUM** — P1 : point bloquant récrit sur C-v016-2-20 tel que versé — vérification dans le **code commité** de la 2a,
    test de comptage par `include_str!` compris ; arrêt et signalement maintenus. P2 : les deux en-têtes de liste récrits
    (`list-col-imported-at`, `list-col-filename`) entrent nommément à `REPLIS_A_SITE_UNIQUE` ; les autres
    `bank-import-labels-*` de l'AC-23 sont réemployées sans changement de texte.
  - **LOW appliqués** — P3 (« d'une même clé »), P4 (borne C-v016-2-20 dans la 2b et la parente), P5 (clé statique sans
    repli littéral : Dev Agent Record avec sa garde, ou issue).
  - **LOW non pris** : aucun.
  Prochaine étape : validation P8 ciblée (`v016-2b-validate-prompt-p8.md`, Opus), dernière passe permise.
- 2026-10-30 — **Validation P8 : clôture au plafond** (ciblée, Opus ; rapport `kesh-gate-logs/v0162b-validate-p8.md`) :
  **0 CRITICAL, 0 HIGH, 1 MEDIUM, 3 LOW**. Huitième passe : la validation se clôt au plafond, selon la règle du
  `CLAUDE.md` (décision de Guy du 2026-10-24) — **aucune remédiation dans la fiche** ; **MEDIUM P8-P1 et LOW P8-P2 à P4
  renvoyés à l'étape 0 du développement**, en tête de « À recontrôler », à déclarer au Change Log ligne lue citée.
  Trend des huit passes de la story (CRITICAL/HIGH/MEDIUM/LOW) : P1 Sonnet, fiche unique, 0/0/12/17 → P2 Opus, fiche
  unique, 0/0/10/17 → (découpage) P3 Sonnet, 2b, 0/0/1/21 → P4 Opus ciblée, 0/0/1/9 → P5 Sonnet ciblée, 0/0/2/5 → P6 Opus
  ciblée, 0/0/2/4 → P7 Sonnet ciblée, 0/0/2/3 → P8 Opus ciblée, 0/0/1/3. Modèles : rotation Sonnet ↔ Opus, contexte frais
  à chaque passe ; aucune passe Haiku. Statut : `ready-for-dev` ; le point bloquant de C-v016-2-20 demeure — le
  développement de la 2b attend le commit du code de la 2a.
- 2026-10-30 — **Développement, étape 0** (Opus 5.5, autonomie ; worktree `/home/gcorbaz/devel/kesh-0163`, branche
  `story/v016-2b-releve-manuel-ecran`, partie de `285d2fde`). Statut `in-progress`.
  - **Point bloquant (C-v016-2-20), vérifié dans le code commité de la 2a** (`git show HEAD:…`, commit `2b14e90f`) —
    **tous les éléments présents**, aucun arrêt : `manual.rs:84` `pub const COUNT: usize = 11;` ; `:87`
    `pub const ALL: [Self; 11] = [` ; `:102` `pub fn index(self) -> usize {`, bras `Self::Required => 0` à
    `Self::UnknownDirection => 10`, aucun `_ =>` ; `:119` `pub fn code(self) -> &'static str {` ;
    `manual_tests.rs:140` `fn all_is_indexed_and_counted() {` (`assert_eq!(ManualStatementReason::ALL.len(),
    ManualStatementReason::COUNT)`, `assert_eq!(r.index(), i, …)`) ; `:160` `let src = include_str!("manual.rs");` dans
    `count_source_variants` ; `:180` `fn all_covers_every_declared_variant() {` (`assert_eq!(n, 11, …)`,
    `assert_eq!(n, ManualStatementReason::COUNT)`, `assert_eq!(n, ManualStatementReason::ALL.len())`). Les numéros de la
    fiche (`:133`, `:153`, `:173`) étaient ceux de l'arbre non commité : décalés de sept lignes au commit, éléments
    identiques.
  - **Renvoi de la validation P8**, appliqué : P1 (MEDIUM) — AC-24, table *Fichiers touchés* (ligne `kesh-i18n`) et liste
    des citations complétées des deux en-têtes et des numéros des quatre catalogues ; la réécriture des catalogues et du
    repli se fait au commit de code. Lignes lues (à `285d2fde`, décalées de +4 en `fr-CH` par les clés de la 2a) :
    `fr-CH:1219` `bank-import-labels-list-col-imported-at = Date import`, `:1220` `… -filename = Fichier` ;
    `de-CH:1158-1159` `Importdatum`, `Datei` ; `it-CH:1158-1159` `Data importazione`, `File` ; `en-CH:1158-1159`
    `Import date`, `File` ; `BankImportList.svelte:24-25`. P2 (LOW) — le Vitest V13
    (`BankImportUpload.test.ts:735` `const FR_TEXTS = [`, `'Date import',` `'Fichier',`, `:845`) se récrit avec les
    nouveaux textes au même commit. P3 (LOW) — AC-24 : « ici, outre les deux de l'AC-23 ». P4 (LOW) — point bloquant :
    « les noms relevés sont indicatifs ».
  - **Autres lignes de « À recontrôler », lues** : `docs/api-external.md:240` (« `GET /companies/current` ne rend que
    les comptes ni archivés ni clos, sans paramètre ») ; `supplier-invoices/import/+page.ts` (`if (browser && role !==
    'Admin' && role !== 'Comptable') { throw redirect(302, '/'); }`) ; `audit-log.spec.ts:52` (`async function
    createUser(page, role)`) ; `reconciliation-amount-due.spec.ts:80-81` (fenêtre de 30 jours) ;
    `textes_coherents.rs:46-48` (`fn lire`) ; `closing-checks-labels.ts:46-49` (repli K2) ;
    `bank-accounts/+page.svelte:836-839` (repli de la boîte de clôture) ; `bank-accounts-page.test.ts:502` (message
    `has-pending-transactions` en simulacre : la 2a l'a récrit, `fr-CH:1443`, le Vitest suit) ;
    `fr-CH:997` (K2), `:1477` (boîte de clôture, +4) ; `de-CH`/`it-CH`/`en-CH:946`, `:1413` (+4).
- 2026-10-31 — **Développement** (Opus 5.5, autonomie ; Dev Agent Record ci-dessus) : rouges d'abord (`b61da862`), écran,
  détail, suppression, catalogues, G24 (`74d61d2c`), manuels et PDF (`3599d922`) ; fusion de la v016-2a revue
  (`c87d0be7`, conflits du registre et de `sprint-status.yaml` résolus par union, choix renumérotés C-v016-2-25 à 28) ;
  409 `RECONCILIATION_ACCOUNT_LOCKED` couvert ; deux gardes rougies au gate corrigées (`67ad0971`). Gate au dernier
  commit de code : nextest 4521/4521 (7 ignorés), Vitest 2200/2200, E2E 327 passés / 7 échecs attendus (KF-029),
  mutations 10/10 tuées, 20 captures. Statut `review`. Prochaine étape : revue de code P1
  (`v016-2b-review-prompt-p1.md`, Sonnet, B/E/A).
- 2026-10-31 — **Revue de code P1** (Sonnet, lentilles B, E, A, contexte frais ; rapports
  `kesh-gate-logs/v0162b-review-p1-{B,E,A}.md`) : **0 CRITICAL, 0 HIGH, 4 MEDIUM, 18 LOW** (B 0/0/2/8, E 0/0/1/6, A 0/0/1/4 ;
  recoupements B2 = E1, B1 = A2, A3 = E3, B4 = E4 = A5, B7 ⊂ A1). Prémisses vérifiées au code avant chaque remède (lignes
  lues : `manual-statement.ts:313-316` `manualSnapshotKey` sans `rowMap` ; `ManualStatementForm.svelte:304-308`
  `screenLines` et son `?? l - 1`, `:401`, `:524`, `:547`, `:562`, `:582` `bg-error-soft`/`bg-warning-soft` ;
  `frontend/src/app.css:12-35` sans `--color-error-soft` ni `--color-warning-soft` ; `jetons-non-definis.test.ts:118-119` ;
  `manual-statement-reason-label.ts:9` (mention dans le commentaire) et `:21` (déclaration) ; `manual.rs:621`
  `currency: "CHF"` ; `user-manual.tex:2359` ; `:141-160` `seq` inatteignable). Traitement (décisions de l'orchestrateur) :
  - **MEDIUM** — B2 = E1 : la table des rangs entre dans `manualSnapshotKey` (signature : le corps construit) ; retirer
    ou ajouter une ligne, vide comprise, périme l'aperçu et le refus ; deux Vitest, mutation **M32**. A1 (et B7) :
    `screenRowsOf`, seule traduction d'une liste de rangs, partagée par `describeManualError` et l'avertissement ; un
    rang inconnu est journalisé (`console.error`) et l'écran dit un texte générique (clé neuve
    `bank-import-manual-warning-duplicate-lines-generic`), « Lignes concernées » omis si aucune ligne ; Vitest de
    formulaire (ligne vide au milieu, doublon après : « … : 3. »), mutation **M31**. B1 = A2 : jetons définis
    (`bg-destructive/10`, `border-destructive`, `text-destructive`, `bg-warning/10`) ; la tolérance de
    `jetons-non-definis.test.ts` **reste**, `error-soft`/`warning-soft` étant encore employés hors de la story
    (`BankImportUpload.svelte`, `bank-accounts/+page.svelte`, `supplier-invoices`, `BankProfileForm.svelte`, le bandeau
    d'erreur antérieur du détail) ; captures 06 et 12 reprises.
  - **LOW appliqués** — A3 = E3 (manuel : « En CHF », aucun refus promis, #233 ; `.tex` et PDF ; l'écran dit « CHF » :
    introduction aux quatre catalogues, libellés des soldes et des colonnes, totaux), B3 (`seq` retiré, la clé tient la
    propriété, en-tête récrit), B4 = E4 = A5 (G24 ancrée sur la déclaration, toute ligne non-entrée est une erreur,
    trois témoins neufs), B5 (refus de création à case : l'aperçu tombe et l'avis « périmé » s'affiche), B6 (succès :
    formulaire figé jusqu'à la navigation, `onSaved` hors du `try`), B8 (notification bornée `/\b3\s+transaction/`), B9
    (échec du chargement des comptes dit, clé neuve `bank-import-manual-load-error` ; Vitest de la page : garde, échec,
    `onSaved`), E2 (solde hors forme marqué `aria-invalid`, « sans séparateur de milliers » aux quatre catalogues), E6
    (`break-all` sur le nom, liste et détail), A4 (AC-21 annotée : 3 Mio), et le texte de l'avertissement « relevé
    identique » complété du numéro (aligné sur le manuel).
  - **LOW non pris, issues demandées** — E5 (aucune garde G13 sur les replis de la famille dynamique
    `bank-import-manual-reason-`, même trou que `bank-import-csv-errors-`), E7 (`amountRegExp` invalide pour
    `maxDecimals: 0` ou `maxIntegerDigits: 0`, latent), B10 (un message de commit de la 2a dit « close », inerte).
  Compteurs recalés : `sitesTotal` 2711 → 2713 (`ManualStatementForm.svelte` 51 → 52, `manual/+page.svelte` 6 → 7) ;
  `REPLIS_A_SITE_UNIQUE` 91 → 93. Gate au dernier commit de code de la remédiation : nextest 4521/4521 (7 ignorés),
  Vitest 186 fichiers 2212/2212, `check` 0 erreur, lint PASS, build vert ; E2E complet 327 passés / 7 échecs attendus
  (KF-029 : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`,
  `:150`) ; mutations **12 jouées, 12 tuées** (M13 à M30, M31, M32) ; captures rejouées (20). Prochaine étape : revue P2
  ciblée (`v016-2b-review-prompt-p2.md`, Opus, une lentille).
- 2026-10-31 — **Revue de code P2** (ciblée, Opus, une lentille ; rapport `kesh-gate-logs/v0162b-review-p2.md`) : **0 CRITICAL,
  0 HIGH, 1 MEDIUM, 6 LOW**. Prémisses vérifiées au code (`ManualStatementForm.svelte:180-184` `previewStale = true` sur un
  refus de création à case, et `:557-562` l'avis « Le relevé a changé » ; `manual-statement.ts:276-277` et `:502-505`, une
  ligne vide en fin ne change pas la table des rangs ; `bank-import-manual-page.test.ts:51` `toThrow()` nu ;
  `boutons-rouges-inventory.test.ts:179` filtre `.test.` ; `textes_coherents.rs` ancre par préfixe). Traitement
  (décisions de l'orchestrateur) :
  - **MEDIUM** — P1 (né de la remédiation B5) : sur `BANK_IMPORT_DUPLICATE_FILE`, `BANK_STATEMENT_DUPLICATE_LINES` ou
    `BANK_STATEMENT_OPENING_BALANCE_GAP` à l'enregistrement, l'écran ne dit plus « le relevé a changé » : le refus porte
    `serverChanged`, et un avis propre (`bank-import-manual-preview-outdated-server`, « La situation du compte a changé
    dans Kesh depuis la vérification : vérifiez de nouveau le relevé, puis confirmez. ») s'affiche avec lui et tombe
    avec lui ; clé aux quatre catalogues, repli égal au fr-CH, `REPLIS_A_SITE_UNIQUE` 93 → 94 ; trois Vitest (un par
    code) ; mutation **M33** (l'ancien avis remis) tuée par les trois.
  - **LOW appliqués** — P2 (en-tête du composant : une ligne vide ajoutée ou retirée en fin de tableau ne périme rien,
    et c'est voulu ; *note : l'entrée « Revue de code P1 » ci-dessus écrit « retirer ou ajouter une ligne, vide comprise,
    périme l'aperçu » — à lire « … une ligne qui déplace une ligne remplie »*), P4 (la garde : redirection exacte
    `{ status: 302, location: '/' }`), P5 (témoin renommé `TemoinFormulaire.test.helper.svelte`, hors des gardes qui
    écartent `.test.`), P7 (G24 : ancre `(?m)^<ouverture>[:= ]`, témoin d'un nom prolongé placé avant la table).
  - **LOW non pris, issues demandées** — P3 (après un succès, l'écran reste figé si la navigation échoue ou si `onSaved`
    manque), P6 (un rang inconnu sur un refus de cellule n'est pas journalisé et devient un refus d'entête).
  Compteurs : `sitesTotal` 2713 → 2714 (`ManualStatementForm.svelte` 52 → 53). Gate au dernier commit de code (`853d23c1`) :
  nextest 4521/4521 (7 ignorés), Vitest 186 fichiers 2215/2215, `check` 0 erreur, lint PASS, build vert ; mutations
  **13 jouées, 13 tuées** (M13 à M33) ; E2E complet **326 passés, 8 échoués, 18 ignorés** : les sept KF-029
  (`mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) et
  `fiscal-year-closing-journey.spec.ts:166` (délai dépassé dans le `beforeEach` `seedTestState`) — **rejouée seule : 2
  passés** (`e2e-rejeu-closing.log`), donc la pollution que `docs/testing.md` § *Les échecs attendus* admet (0 à 2), sans
  rapport avec le diff. Prochaine étape : revue P3 ciblée (`v016-2b-review-prompt-p3.md`, Sonnet).


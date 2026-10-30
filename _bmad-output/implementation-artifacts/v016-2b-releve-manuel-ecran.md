# Story v016-2b : saisir un relevé bancaire à la main — l'écran

Status: ready-for-dev

*Sous-fiche issue du découpage de la Story v016-2 à la validation P2 (2026-10-30, décision de l'orchestrateur,
C-v016-2-18). **Dépend de la `v016-2a`** (l'API, les codes, l'audit, la migration) : elle se développe après sa fusion
sur la branche. Worktree `/home/gcorbaz/devel/kesh-0162`, branche `story/v016-2-releve-manuel` (partie de `c5101b07`), la 2a d'abord,
puis la 2b. Registre **commun** : `v016-2-choix-autonomes.md` (C-v016-2-1 à C-v016-2-18). Issue : **#816**, en `refs`
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
  à côté du bouton : **une seule**, la première dans l'ordre de cette liste à partir de « aucun compte », puis :
  période, soldes, lignes (la première ligne fautive), écart, aperçu, cases. *Vérifier* est actif dès qu'un compte et
  une période lisible sont saisis, même si l'écart n'est pas nul ou qu'une ligne est fautive — si bien que les erreurs
  du serveur de l'AC-22 sont atteintes par l'usage normal. Pour M14, le test pose un **aperçu réussi** sur un formulaire
  déséquilibré et attend la raison « écart ». *(Validation P3, R6 :)* la date de valeur n'est pas contrôlée par le
  client (un `<input type="date">` la rend lisible ou vide) ; une faute de serveur désigne **la cellule** (ligne et
  colonne, `field` traduit par l'en-tête de colonne déjà affiché) et le message est le libellé du `reason`. Le format
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
  `submitting` (patron `DeleteBankImportAction.svelte`).
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
  et les textes d'interface qui disent « importée » (validation P3, F4) — `homepage-bank-no-transactions` (`:166`),
  `bank-accounts-errors-has-pending-transactions` (`:1439`), `bank-accounts-errors-has-transactions` (`:1433`),
  `bank-accounts-close-dialog-body` (`:1473`) — se récrivent pour valoir aussi d'un relevé saisi, comme le manuel
  (AC-27). Le refus « relevé identique » (`BANK_IMPORT_DUPLICATE_FILE`) porte le message serveur « Ce fichier a
  déjà été importé » (`errors.rs:2354-2358`) : l'écran de saisie affiche sa propre clé `bank-import-manual-*` et ne
  montre pas ce message.
- **AC-24** — i18n de l'écran : toutes les clés nouvelles du frontend (`bank-import-manual-*`, `bank-import-detail-*`,
  libellés des `reason`) dans
  les **quatre** catalogues (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`), aucun texte en dur dans le
  balisage ; `npm run lint-i18n-ownership` vert (préfixe `bank-import-` admis pour `features/bank-import/`, aucune clé
  d'un autre domaine réutilisée — pas de `journal-entry-form-*`). Gardes qui rougiront et se recomptent depuis la
  source (validation P1, F8) : `frontend/src/lib/shared/i18n-keys.test.ts` (`ATTENDU.sitesTotal`, cardinalité exacte),
  `i18n-un-repli-par-cle.test.ts`, `i18n-repli-divergent-actif.test.ts` (repli = valeur `fr-CH`),
  `i18n-libelle-en-dur.test.ts`, `MOTIFS_DYNAMIQUES` et `CARDINALITES` (famille `bank-import-manual-reason-`), et la
  parité des catalogues (`crates/kesh-i18n/src/loader.rs`, `parity_between_locales`). *(Les clés des messages serveur
  et du libellé d'audit sont posées par la 2a, AC-11 et AC-15.)* *(Validation P3, R3 :)* compteurs à recompter, non à
  ajuster : `i18n-keys.test.ts:773` (`sitesGabarit`, 11 → 12), `:960` (`SITES_GABARIT_ATTENDUS`, une ligne neuve),
  `i18n-libelle-en-dur.test.ts:250` (`CANDIDATES_ATTENDUES`, 71 → à recompter si la table expose une fonction
  `…Label`), et `i18n-entrees-a-variables.test.ts` (clés à variables). **Test croisé des `reason`** (décision de
  l'orchestrateur à la validation P3, R1) : la table TypeScript des `reason` (fichier de `features/bank-import/`) est
  lue par un **test Rust** (`crates/kesh-api/tests/`, patron `lire` de `textes_coherents.rs:46-48`), qui extrait ses
  clés et les égale, dans les deux sens, à `ManualStatementReason::ALL` mappé par `code()` (2a, AC-4) ; le Vitest de
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
  **compte bancaire propre à l'exécution** (`wiredBankAccountViaApi` avec un IBAN suisse synthétique **unique**, clé
  MOD-97 calculée, **et `isPrimary: false`** — l'aide crée aujourd'hui un compte principal
  (`helpers/api-fixtures.ts:190`), ce qui ferait d'un compte neuf le principal de la société du seed à chaque
  exécution : elle gagne un paramètre `isPrimary`, défaut `true`, `false` ici ; validation P2, R11), si bien qu'aucun relevé antérieur n'existe et qu'aucun avertissement de solde de départ ne peut
  surgir d'une exécution à l'autre ; ouvre `/bank-import`, suit le lien, saisit un relevé de **trois** lignes dont un crédit égal au TTC de la
  facture, constate l'écart, corrige le solde d'arrivée, *Vérifier*, *Enregistrer* ; retrouve le relevé « Saisie
  manuelle » dans la liste ; ouvre `/reconciliation`, choisit le compte, accepte la proposition de la facture ; ouvre
  `/audit-log`, filtre sur l'action « Relevé bancaire saisi à la main » **et** sur le type et l'identifiant de l'entité
  (l'import créé), et retrouve l'entrée. L'IBAN synthétique est produit par une aide `helpers/` qui calcule la clé MOD-97
  (validée côté serveur par `kesh-qrbill/src/validation.rs:40`, CH/LI, 21 caractères). Un second test : un
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
`frontend/tests/e2e/bank-manual-statement.spec.ts`, `frontend/tests/e2e/helpers/api-fixtures.ts` (`isPrimary`, `createUser`
extraite, IBAN MOD-97) ; `crates/kesh-api/tests/` (test croisé des `reason`, AC-24).

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
| `frontend` | `src/lib/features/bank-import/{ManualStatementForm.svelte, manual-statement.ts, *.test.ts, bank-import.api.ts, bank-import.types.ts, BankImportList.svelte, DeleteBankImportAction.svelte}`, la table des `reason`, `src/lib/features/journal-entries/balance.ts`, `src/routes/(app)/bank-import/+page.svelte`, `src/routes/(app)/bank-import/manual/+page.svelte` (+ `+page.ts`, garde), `src/routes/(app)/bank-import/[id]/+page.svelte`, `src/lib/shared/i18n-keys.test.ts`, `src/lib/shared/i18n-libelle-en-dur.test.ts`, `tests/e2e/bank-manual-statement.spec.ts`, `tests/e2e/helpers/api-fixtures.ts` (et une aide `createUser`, une aide d'IBAN), `tests/e2e/audit-log.spec.ts` (`createUser` extraite) |
| `kesh-api` (tests) | le test croisé des `reason` (AC-24) |
| `kesh-i18n` | `locales/{fr,de,it,en}-CH/messages.ftl` (clés de l'écran ; `:166`, `:1433`, `:1439`, `:1473` récrites) |
| docs | `docs/manual/fr/user-manual.tex` (+ PDF), `docs/manual/fr/admin-manual.tex:82` (+ PDF), `CHANGELOG.md` (*Ajouté*), `README.md:41` |

## À recontrôler à l'étape 0 du développement

- Que `GET /companies/current` ne rend que les comptes ni archivés ni clos (`docs/api-external.md:240`).
- `frontend/src/routes/(app)/supplier-invoices/import/+page.ts` — le patron de garde Comptable+ (`load()` redirige un rôle
  autre que Admin/Comptable vers `/`), à reprendre pour `/bank-import/manual` *(les deux puces jumelles fusionnées à la
  validation P3, R9 = F8)*.
- Les citations des AC (ajoutées à la validation P3, R9 = F8) : `[id]/+page.svelte:37`, `:62`, `:74`, `:76`, `:88`, `:92`,
  `:113-118` ; `BankImportList.svelte:24-25` ; `helpers/api-fixtures.ts:100-105`, `:190` ; `audit-log.spec.ts:52` ;
  `reconciliation-amount-due.spec.ts:80-81` ; `kesh-core/src/bank_imports.rs:365-377` ;
  `kesh-import/src/camt053/mod.rs:405`, `:412`, `:426-428`, `:629-630` ; `errors.rs:2354-2358` ;
  `i18n-keys.test.ts:773`, `:882`, `:960` ; `i18n-libelle-en-dur.test.ts:250` ; `messages.ftl:166`, `:1433`, `:1439`,
  `:1473` ; `user-manual.tex:1971-1973`, `:2277`.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

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

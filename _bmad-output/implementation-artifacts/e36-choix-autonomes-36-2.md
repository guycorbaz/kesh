# Epic 36 — registre des choix autonomes de la story 36-2 (#1150)

Registre propre à la story 36-2, **versé par union**. Fiche : `36-2-eclater-bouton-grise.md`. Code lu à `dabd9045a`.
Ces choix ne sont pas couverts par C-e360 ; ils prolongent l'étape 0 de la 36-2 et C-e360-5 (registre
`e36-choix-autonomes-e360.md`), qui font foi : la cause n'est pas tranchée par la fiche, la reproduction la départage.
*(Annoté le 2026-11-14, validation P2 : depuis C-36-2-9, la reproduction constate la cause sans décider du contenu des
correctifs ; elle peut en suspendre la mise en œuvre — fiche, *Si R0 rend…*. C-e360-5 est annoté en ce sens par
l'orchestrateur au registre du kickoff.)*

### C-36-2-1 — H4 (le catalogue efface l'attendu et l'écart) se corrige sans condition

- **Contexte** : l'étape 0 nomme H1 à H3. La lecture du catalogue (`fr-CH/messages.ftl:1620-1621`) contre
  `i18n.svelte.ts:20` montre un quatrième fait, établi et non supposé : en production, l'indicateur dit « Balance » et le
  message du déséquilibre ne dit ni l'attendu ni l'écart.
- **Retenu** : correction dans la story (D3), parce que l'attendu de #1150 est « ou l'écran dit ce qui manque ».
- **Écartée** : une issue séparée — la story livrerait un écran qui ne dit toujours pas ce qui manque dans le cas le plus
  probable d'un bouton grisé sur des lignes remplies.
- **Réversible** : oui.

### C-36-2-2 — H5 (le montant part en nombre JSON, refusé par `serde-str`) se corrige sans condition

- **Contexte** : `bind:value` sur `type="number"` rend un nombre (Svelte 5.55.1, `bindings/input.js:31`, `:280-289`) ;
  `Decimal` sous `serde-str` exige une chaîne (`rust_decimal-1.43.0/src/serde.rs:289-296`, `serde_json-1.0.154/src/de.rs:1535-1544`).
  Aucun test ne fait traverser la frontière HTTP à un éclatement saisi à l'écran.
- **Retenu** : le montant part en chaîne (D2), prouvé par un Vitest et par l'E2E R0 (a), qui passe l'éclatement de bout en
  bout.
- **Écartée** : une issue séparée — le bouton activé mènerait à un refus, et #1150 resterait vraie en substance.
- **Réversible** : oui.

### C-36-2-3 — Branche H2 : la grammaire du journal, importée, non modifiée

- **Contexte** : si Chromium fr-CH jette une saisie (`70,00`, `1'200.00`), le champ montre un texte que la valeur ignore.
- **Retenu** : champ texte lu par `isValidAmount` / `parseAmount` de `journal-entries/balance.ts`, sans le modifier ; la
  36-3 (C-e360-6) en fixe l'évolution, dont le dialogue hérite à la fusion.
- **Écartées** : `valueAsNumber` (la saisie jetée reste muette) ; la grammaire du solde d'ouverture (`bank-accounts/format.ts`) ;
  une grammaire propre au dialogue.
- **Réversible** : oui.
- *(Annoté le 2026-11-14, remédiation de la validation P1 : **remplacé par C-36-2-9** — le champ devient texte sans
  condition, lu par `readJournalAmount` de la 36-3.)*

### C-36-2-4 — R0 se joue aussi contre l'image 0.17.6

- **Contexte** : Guy a vu le défaut le 2026-11-13, en 0.17.6 ; la 35-7 (0.17.7) a changé le `blur` du champ strict, si bien
  qu'une reproduction à la tête seule peut ne plus montrer H1.
- **Retenu** : R0-b rejoue la même spec contre `gcorbaz/kesh:0.17.6` (image présente sur la station), sur une base neuve de
  la voie 2 ; si l'image ne peut pas la jouer, le Record le dit et s'en tient à la lecture de `v0.17.6`.
- **Écartée** : la lecture seule de `v0.17.6` (elle ne dit rien de H2 ni de H3).
- **Réversible** : oui.

### C-36-2-5 — Branche H1 : le champ de compte n'est pas touché

- **Contexte** : à la tête, le `blur` d'un champ strict jette le brouillon (35-7) et le message dit « compte requis » ; un
  numéro exact puis *Tab* qui choisirait le compte est une autre décision (#1084), sur sept consommateurs.
- **Retenu** : aucun changement d'`AccountAutocomplete.svelte` ; si R0-a contredit la lecture, arrêt et amendement de la fiche.
- **Écartée** : choisir l'unique correspondance exacte au `blur` (contraire à D2 de la 35-7, « seul un choix explicite
  sélectionne », et décision qui revient à Guy dans #1084).
- **Réversible** : oui.

### C-36-2-6 — Les clés de l'indicateur : la clé existante pour l'état équilibré, une clé neuve pour l'écart

- **Contexte** : `reconciliation-split-balance-indicator` sert aujourd'hui les deux états (`TransactionSplitModal.svelte:405`,
  `:410`) avec deux replis différents.
- **Retenu** : la clé existante garde l'état équilibré (`{ $expected }`) ; `reconciliation-split-balance-gap` neuve porte
  l'écart ; `reconciliation-split-error-imbalance` reçoit `{ $expected }` et `{ $diff }`.
- **Écartée** : retirer la clé partagée et en créer deux (plus de lignes touchées dans quatre fichiers, sans gain).
- **Réversible** : oui.
- *(Annoté le 2026-11-14, remédiation de la validation P1 : **remplacé par C-36-2-7** pour
  `reconciliation-split-error-imbalance`, qui reste intacte ; le reste — clé existante pour l'état équilibré, clé neuve
  `-balance-gap` — est maintenu.)*

---

*Choix suivants : décisions de l'orchestrateur sur la validation P1 (`kesh-gate-logs/e36-2/validation-p1.md`), consignées
le 2026-11-14 ; prémisses relues au code avant application.*

### C-36-2-7 — Le message du dialogue prend une clé neuve ; `reconciliation-split-error-imbalance` reste intacte (V1-1)

- **Contexte** : la clé est lue sans arguments par `crates/kesh-api/src/errors.rs:3430` (message de la
  `400 RECONCILIATION_SPLIT_IMBALANCE`) et `failed-proposal-label.ts:379` (échec partiel) ; lui donner des variables
  changerait le message de l'API et afficherait `{ $expected }` dans l'étiquette.
- **Retenu** (décision de l'orchestrateur) : clé neuve `reconciliation-split-error-imbalance-detail` (`{ $expected }`,
  `{ $diff }`) pour le dialogue ; l'ancienne intacte dans les quatre locales ; garde au test de catalogue ; grep des
  consommateurs Rust et TS pour toute clé dont la valeur change.
- **Écartées** : variables sur l'ancienne clé ; chiffres passés aussi par l'API et l'étiquette (changement de contrat).
- **Réversible** : oui.

### C-36-2-8 — La 36-2 se développe à partir de la 36-1 et de la 36-3 fusionnées (V1-2, V1-4)

- **Contexte** : le lecteur `readJournalAmount` naît dans la 36-3 (fiche `36-3-ecriture-montant-ambigu.md`, D2).
- **Retenu** (décision de l'orchestrateur) : la branche se refonde sur la lignée A après la fusion de la 36-3, avant tout
  correctif ; R0 peut s'écrire et se jouer avant.
- **Écartée** : développer en parallèle de la 36-3 avec la grammaire d'aujourd'hui (deux lectures du montant à fusionner).
- **Réversible** : oui (au prix d'un correctif sur `isValidAmount` / `parseAmount`).

### C-36-2-9 — Aucun correctif ne dépend plus du verdict de R0 ; H2 n'est plus une branche (V1-2, V1-4)

- **Contexte** : les branches H2/H3 n'étaient pas bornées ; H5 réfutée et « aucune hypothèse reproduite » n'étaient pas
  écrits.
- **Retenu** (décision de l'orchestrateur) : champ de montant texte sans condition, lu par `readJournalAmount`, charge en
  chaîne normalisée ; messages tranchés (vide, `0`, `0,00` → « montant > 0 requis » ; `-70.00`, `7O` → « illisible » ;
  `1.234` → « ambigu ») ; table *Si R0 rend…* : H5 réfutée → la conversion reste, phrase retirée de la note, #1150 non
  fermée sur ce motif ; aucune hypothèse reproduite → arrêt après R0 et rapport, sans correctif inventé, `refs #1150` ;
  H1 contredite → arrêt.
- **Écartées** : garder H2 comme branche (critère flou, correctif partiel) ; corriger sans cause établie.
- **Réversible** : oui.
- *(Annoté le 2026-11-14, validation P4 : **remplacé, pour la mise en œuvre, par C-36-2-15** — sans grisé reproduit,
  D2 à D6 se développent quand même (table A, A3) ; seul `refs #1150` demeure (table B, B3). Le reste — champ texte sans
  condition, messages tranchés, H5 réfutée — est maintenu.)*

### C-36-2-10 — Localisateurs de R0 indépendants de D4 (V1-3)

- **Retenu** : R0 ne s'appuie que sur des `data-testid` présents à la tête et en 0.17.6 et localise le message par son
  texte ; passage à `split-client-error` au commit de D4.
- **Écartée** : `split-client-error` dès R0 (rouge par absence du `testid`, relevé vide, rien de départagé).
- **Réversible** : oui.

### C-36-2-11 — Pas de `role="status"` sur le message (V1-5)

- **Contexte** : l'indicateur voisin porte `aria-live="polite"` (`TransactionSplitModal.svelte:401`) et dit le déséquilibre.
- **Retenu** : `data-testid` seul ; pas de seconde région vivante.
- **Écartée** : `role="status"` (double annonce du déséquilibre, sans test qui l'exerce).
- **Réversible** : oui.

### C-36-2-12 — #1151 entre sans condition, sur le précédent `LinkSettlementModal` (V1-4, V1-6)

- **Contexte** : la passe juge H3 quasi certaine (quatre à cinq colonnes dans ≈ 22rem).
- **Retenu** (décision de l'orchestrateur) : `class="max-h-[85vh] overflow-y-auto sm:max-w-3xl"` sur le seul
  `Dialog.Content` du dialogue (`LinkSettlementModal.svelte:195`) ; `closes #1151` à la PR.
- **Écartée** : garder la condition (C-e360-5 le permettait, la mesure la rend sans objet).
- **Réversible** : oui.
- *(Annoté le 2026-11-14, validation P2 : la classe est **remplacée par C-36-2-14** — `sm:max-w-3xl` seul, sans
  `max-h` ni `overflow-y-auto` ; l'entrée de #1151 sans condition est maintenue.)*

### C-36-2-13 — Garde du centime dans le dialogue ; les zéros au-delà du centime se normalisent (V2-1)

- **Contexte** : le serveur refuse une échelle supérieure à 2 (`crates/kesh-api/src/routes/reconciliation.rs:5369-5373`) ;
  la grammaire du journal (36-3, D2) admet quatre décimales et `payload` garde la fraction telle que saisie.
- **Retenu** (décision de l'orchestrateur, forme choisie ici) : une lecture `clear` est acceptée si `value` égale son
  arrondi au centime, et part `value.toFixed(2)` (« 60.0000 » → `60.00`) ; sinon, clé neuve
  `reconciliation-split-error-amount-cents`. Textes des clés neuves sans conseil de quatre décimales. `readJournalAmount`
  non modifié.
- **Écartées** : refuser aussi « 60.0000 » (zéros sans valeur, lecture exacte) ; laisser refuser le serveur (message
  technique, index à partir de 0) ; modifier `readJournalAmount`.
- **Réversible** : oui.

### C-36-2-14 — La largeur sans défilement : la liste du champ de compte n'est pas rognée (V2-2)

- **Contexte** : la liste d'`AccountAutocomplete` est un descendant en `absolute` (`:459`, `:555`) ; un ancêtre
  `overflow-y-auto` la rogne. Aucun dialogue défilant ne loge ce composant aujourd'hui.
- **Retenu** : `class="sm:max-w-3xl"` seul sur le `Dialog.Content` ; assertion `toBeInViewport` d'une option de la liste
  ouverte sur la dernière de six lignes (AC6 b), mutation M9 ; si l'assertion échoue : arrêt de cette partie et rapport.
  La limite des dialogues très longs (bas hors de l'écran, comme aujourd'hui) est une issue demandée (I3).
- **Écartées** : défilement du seul corps des lignes (rogne davantage) ; liste rendue hors du conteneur (modifie
  `AccountAutocomplete.svelte`, hors de la story) ; `max-h-[85vh] overflow-y-auto` du précédent.
- **Réversible** : oui.
- *(Annoté le 2026-11-14, validation P3 : la classe devient `sm:max-w-[min(48rem,calc(100%-2rem))]`, C-36-2-16 ; I3 est
  ouverte : #1181.)* *(Annoté le 2026-11-14, validation P4 : C-36-2-16 retiré, **`sm:max-w-3xl` seul rétabli**.)*

---

*Choix suivants : validation P3 (`kesh-gate-logs/e36-2/validation-p3.md`), consignés le 2026-11-14.*

### C-36-2-15 — *Si R0 rend…* : deux tables à cas exclusifs ; les correctifs ne dépendent pas de R0-b (V3-1)

- **Contexte** : la table unique se contredisait (H5 constatée → développer ; R0-b impossible → suspendre) sans
  précédence, et « lignes que Guy jugerait complètes » n'était pas constatable.
- **Retenu** (décision de l'orchestrateur, forme choisie ici) : table A pour D2 à D6 — A1 H1 contredite à la tête → arrêt ;
  A2 R0-a impossible → arrêt ; A3 sinon → développés ; table B pour #1150 — B1 grisé reproduit à la tête → `closes`
  (après vérification que la même variante active le bouton après correctif) ; B2 en 0.17.6 seulement → `closes` ;
  B3 nulle part → `refs`, aucun correctif de plus. Première ligne applicable seule. « Grisé reproduit » : chaque ligne
  montre un compte et un montant tapé, somme tapée = montant de la transaction, `split-submit` désactivé. Cette règle
  remplace, pour la mise en œuvre, la suspension écrite à C-36-2-9.
- **Écartée** : suspendre D2 à D6 sans grisé reproduit (fait dépendre de R0-b des défauts établis à la lecture).
- **Réversible** : oui.
- *(Annoté le 2026-11-14, validation P4 : critère rendu mécanique — variantes éligibles (a) à (d) seulement, montants
  admis = leur liste fermée (positifs, deux décimales au plus, non ambigus), somme comparée à la **valeur absolue** du
  montant de la transaction ; B2 scindée par variante (B2-ter : `refs`) ; table B sans objet sous A1 ou A2.)*

### C-36-2-16 — La largeur bornée à toute taille de fenêtre (V3-5)

- **Contexte** : `dialog-content.svelte:31` porte `max-w-[calc(100%-2rem)]` et `sm:max-w-sm` ; `sm:max-w-3xl` remplace
  le second, et la borne ne vaut plus dès 640 px : de 640 à ≈ 800 px, 768 px débordent.
- **Retenu** (décision de l'orchestrateur, corrigé dans la story) : `sm:max-w-[min(48rem,calc(100%-2rem))]` ; AC6 c à
  700×720, R0 (i), capture, M10.
- **Écartées** : `sm:max-w-3xl` seul (le défaut du précédent) ; corriger aussi `LinkSettlementModal` (hors périmètre).
- **Réversible** : oui.
- *(Annoté le 2026-11-14, validation P4 : **remplacé — retiré**. La prémisse était fausse : le dialogue est `fixed` et
  `w-full` (`dialog-content.svelte:31`), `max-w` ne fait que plafonner une largeur égale à la fenêtre, et rien ne déborde ;
  seule la marge de 1rem se perd entre 640 et 800 px, comme au précédent. `sm:max-w-3xl` seul est rétabli ; AC6 c, R0 (i)
  et M10 sont retirés.)*

### C-36-2-17 — Les dossiers de R0-b hors du worktree, sous `/tmp/kesh-voie2/` (V3-6)

- **Contexte** : l'image n'a pas de directive `USER` ; le conteneur écrit en root.
- **Retenu** : `-v /tmp/kesh-voie2/e36-2-r0b:/data` (le `TMPDIR` de la voie 2), vidé au redémarrage ; rien de root dans
  le worktree, que `git worktree remove` doit pouvoir retirer.
- **Écartée** : `--user "$(id -u):$(id -g)"` (image non éprouvée sans root ; un refus d'écriture hors de `/data` ferait
  échouer R0-b sans rapport avec la story).
- **Réversible** : oui.
- *(Annoté le 2026-11-14, validation P4 : un dossier neuf par essai, `/tmp/kesh-voie2/e36-2-r0b-<n>/` — les fichiers
  root d'un essai ne s'effacent pas sans `sudo`, et un second essai ne doit pas remonter ceux du premier.)*

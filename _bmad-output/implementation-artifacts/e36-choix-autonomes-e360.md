# Epic 36 — registre des choix autonomes du kickoff (clé `e360`)

Registre propre au kickoff (story 36-0), **versé par union** par l'orchestrateur. Chaque choix : contexte, option
retenue, options écartées, réversibilité. Lignes citées sur `main` = `42d5f368` (rétrospective de l'Epic 35, après le
tag v0.17.7). Fichier d'epic : `_bmad-output/planning-artifacts/epic-36-premiere-facture-et-suite.md`.

---

## Étape 0 — ce que le code dit de chaque story de la phase 1

Lecture faite au kickoff : l'issue (`gh issue view N --json title,labels,body,comments` ; `gh issue view N --comments`
échoue sur ce dépôt, erreur GraphQL des *Projects (classic)*) et le code qu'elle cite. Aucune fiche n'est rédigée ici.

### 36-1 — #1095, *Entrée* dans le champ « compte de produit »

- **Constat.** `AccountAutocomplete.svelte:265-270` n'intercepte *Entrée* que si la liste est ouverte **et** a une
  correspondance (`if (open && filtered[highlightIndex])`) ; sinon l'événement suit son cours et soumet le `<form>`
  englobant **sans `blur`**, donc sans passer par `handleBlur` (`:283-317`), qui seul jette le brouillon. Le champ
  montre alors `draft` (`query = draft ?? displayLabel`, `:180`) pendant que `value` porte l'ancien compte. Les deux
  consommateurs atteints sont les seuls placés dans un `<form>` : `InvoiceForm.svelte:697` (`<form onsubmit>`), ligne de
  facture `:877-881` (`allowClear`) ; `products/+page.svelte:622-627` (`<form onsubmit>`), compte de produit `:697-703`
  (`allowClear`). Inventaire des autres consommateurs (`grep -rn "<AccountAutocomplete"`) : `ManualMatchModal.svelte:188`,
  `TransactionSplitModal.svelte:303`, `VatPurchaseAssistant.svelte:159`, `:204`, `JournalEntryForm.svelte:397` — aucun
  dans un `<form>` (`grep -n "<form"` vide sur chacun) ; le `Ctrl+S` de `JournalEntryForm` rend déjà le focus (35-7, E1,
  commentaire `:289`).
- **Taille.** Un composant et deux consommateurs, frontend seul : `features/journal-entries`, `components/invoices`,
  `routes/(app)/products` — trois modules ; tests Vitest par consommateur, une spec E2E (facture). **Petite.**
- **Splitting préventif** : ne s'applique pas (trois modules ; pas encore de passe). **Migration** : aucune.

### 36-2 — #1150, *Éclater* reste grisé

- **Constat.** Le bouton est désactivé par `disabled={submitting || !!clientError}` (`TransactionSplitModal.svelte:452`) ;
  `clientError` (`:162-214`) rend la **première** règle violée : compte nul (`:183-189`), montant non lu ou nul
  (`parseFloat`, `:190-197`), description trop longue, puis l'équilibre (`balanceExact`, `:148-153`, sur une somme
  `parseFloat` arrondie au centime, `:136-146`). Le message s'affiche sous le tableau (`:432-438`). **La cause n'est pas
  établie à la lecture** ; trois hypothèses, à départager par reproduction avant toute fiche définitive :
  - **H1** — le compte tapé sans être choisi dans la liste : `AccountAutocomplete` est **strict** ici (pas d'`allowClear`,
    `:303-308`), et `handleBlur` jette le brouillon d'un champ strict (`AccountAutocomplete.svelte:300-303`, 35-7,
    C-e350-37) : un numéro tapé puis *Tab* laisse `counterpartyAccountId` à `null` → « Ligne N : compte requis ». Même
    classe que #1095 ;
  - **H2** — le montant : `<Input type="number" step="0.01" bind:value>` (`:311-316`) rend une valeur vide pour une saisie
    que le navigateur ne lit pas (apostrophe, virgule selon la locale) → « montant > 0 requis » ;
  - **H3** — le message existe mais ne se voit pas : #1151 (Guy, même jour) dit la fenêtre « trop étroite, les champs ne
    se lisent pas ».
  L'API réussit (un agent l'a fait par clé) : le serveur n'est pas en cause. Tests existants :
  `TransactionSplitModal.test.ts` (sélection par la liste, `:251-254`), `tests/e2e/reconciliation-split.spec.ts`.
- **Taille.** Un ou deux fichiers frontend (+ `AccountAutocomplete.svelte` si H1). **Petite**, sous réserve de la cause.
- **Splitting préventif** : ne s'applique pas. **Migration** : aucune.

### 36-3 — #1051, « 1.234 » à l'écriture manuelle

- **Constat.** `JournalEntryForm.svelte:208-209` envoie `line.debit.replace(',', '.')` ; l'équilibre et la validité
  passent par `balance.ts` : `amountRegExp` (`:50-53`, `^\d{1,15}([.,]\d{1,4})?$`), `isValidAmount` (`:61-64`),
  `parseAmount` (`:68-71`). « 1.234 » y est valide et vaut 1,234. La règle de la 35-6 est `openingAmountKind`
  (`bank-accounts/format.ts:280-332`) : **sa grammaire ne se réemploie pas** (elle admet signes, apostrophes et blancs
  de groupe, souligné — `:244-252`, `:283-307` —, que le journal refuse) ; **son prédicat d'ambiguïté se réemploie**
  (`:321-328` : un séparateur décimal, trois chiffres après, partie entière non groupée de un à trois chiffres hors
  zéros de tête). ⚠️ Le message de la 35-6 (`messages.ftl` fr-CH `:1780`) conseille « Écrivez 1'234 » : le journal
  **refuse** l'apostrophe (`amountRegExp`) — il faut un message propre à ce format, ou admettre l'apostrophe au journal
  (changement de grammaire, plus large). Le serveur n'est pas modifié (`Decimal::from_str`, `routes/journal_entries.rs`,
  second commentaire de #1051 : C-e350-19 l'a laissé tel quel).
  **Même symptôme ailleurs** (`grep -rn "isValidAmount(\|parseAmount("`) : la page des soldes d'ouverture
  (`routes/(app)/settings/opening-balances/+page.svelte:150-171`, `:490-634`), l'assistant d'achat avec TVA
  (`VatPurchaseAssistant.svelte:73-75`), le prix unitaire des lignes de facture fournisseur
  (`supplier-invoice-form.ts:78`, `:120`, `:152-153`) ; la **quantité** (`:63`, `:152`) lit légitimement « 1.234 »
  (1,234 unité) et doit rester hors de la règle ; le relevé manuel (`manual-statement.ts:118`, deux décimales) refuse
  déjà « 1.234 ».
- **Taille.** Écriture manuelle seule : `features/journal-entries` (+ prédicat extrait vers un utilitaire partagé),
  `messages.ftl` ×4, manuel — quatre à cinq modules. Étendue aux soldes d'ouverture et aux factures fournisseur : plus
  de cinq.
- **Splitting préventif** : **à la limite** ; voir C-e360-6. **Migration** : aucune.

### 36-4 — #1131, la clé d'API perdue à l'annulation

- **Constat.** La prémisse de l'issue est **en partie dépassée** sur `main` : la route
  `post_cancel_reconciliation` (`routes/reconciliation.rs:5947-5969`) lit `current_user.api_key_id` (`:5955`) et le passe
  à `cancel_in_tx` (`:5999-6007`) ; l'audit `reconciliation.cancelled` le porte (`reconciliation_cancel.rs:797-801`,
  `for_actor` à `:799`), et la contre-passation d'une écriture rapprochée seule aussi (`:741-751`, 35-8a2). L'enveloppe
  `reconciliation_cancel::cancel` (`:831-858`), qui passe `None`, n'a **aucun appelant de production**
  (`grep -rln "reconciliation_cancel::cancel(" crates/*/src` vide ; appelée par cinq fichiers de tests de `kesh-db`). **Ce qui perd la
  clé** : les deux annulations de règlement appelées par le dé-rapprochement, qui ne reçoivent que `user_id` —
  `invoice_settlements_write::cancel_settlement_in_tx` (`reconciliation_cancel.rs:713-720` →
  `invoice_settlements_write.rs:917-924`) et `supplier_invoices::cancel_settlement_in_tx` (`:731-737` →
  `supplier_invoices.rs:2815`) —, donc leurs audits (`invoice.settlement_cancelled`, `journal_entry.reversed`,
  `supplier_invoice.settlement_cancelled`, délettrage). Les mêmes fonctions servent les routes d'annulation d'un
  règlement (`routes/invoices.rs:1387-1405`, `routes/supplier_invoices.rs:1015-1029`, par les enveloppes
  `invoice_settlements_write.rs:1142` et `supplier_invoices.rs:3003`). La garde de la 35-8a1 les inventorie
  `AConvertir` (`tests/audit_acteur_garde.rs:294-299`, `:356-361`). `docs/api-external.md:404` dit encore « la clé n'y
  figure que pour le rapprochement […] dé-rapprochement compris » (V1-3 de la validation de la 35-8b).
- **Taille.** `kesh-db` (trois dépôts), `kesh-api` (deux routes), la garde, ≈ 15 fichiers de tests appelant
  `cancel_settlement` (`grep -rln "cancel_settlement(" crates/*/tests | wc -l` → 15), `api-external.md` et les manuels
  si la phrase y figure. Rollout mécanique du patron `Acteur` (35-8a1, 35-8a2). **Petite à moyenne.**
- **Splitting préventif** : cinq modules au plus, patron établi : ne s'applique pas, à recontrôler à la fiche.
  **`kesh-db` touché** : gate complet même en cours de boucle. **Migration** : aucune (colonnes de `20260605000002`).

### 36-5 — #931 (KF-056), le compte rendu de la restauration perdu

- **Constat.** Cause écrite sur l'issue (commentaire du 2026-11-13) et cohérente avec le code : `sessionIdle` démarre
  au layout racine (`routes/+layout.svelte:36`), donc aussi sous l'assistant ; `tick` (`session-idle.svelte.ts:326-349`)
  envoie un battement `POST /api/v1/auth/activity` (`:351-377`) dès qu'une activité non rapportée existe — le clic de
  confirmation en est une. Si l'import (`AdminRestorePanel.svelte:143-165`, `uploadFullImport` `:156`) invalide la
  session avant ce battement, celui-ci rend 401, le chemin 401 d'`api-client.ts` (`refreshTokens`, `:198-203`) efface la
  session et redirige vers `/login?reason=session_expired` : le panneau qui dit les pièces manquantes disparaît. Le
  même composant sert `/admin/restore` (`routes/(app)/admin/restore/+page.svelte:7-10`) : **un utilisateur réel** peut
  perdre le compte rendu. Specs touchées : `backup-restore-new-install.spec.ts:140`, `:262`. KF-056 n'est pas listée
  dans `docs/testing.md` (`grep -n "KF-056\|931"` vide).
- **Taille.** `session-idle.svelte.ts` et `AdminRestorePanel.svelte` (peut-être `api-client.ts`), leurs tests, la spec
  E2E, le manuel si le geste visible change : frontend seul, deux à trois modules. **Petite.**
- **Splitting préventif** : ne s'applique pas. **Migration** : aucune.

---

### C-e360-1 — Clés et ordre de la phase 1 : l'ordre de Guy

- **Contexte** : Guy a listé #1095, #1150, #1051, #1131, #931 ; le critère d'impact mettrait #1051 (I1) avant #1150 (I2).
- **Retenu** : 36-1 #1095, 36-2 #1150, 36-3 #1051, 36-4 #1131, 36-5 #931 — l'ordre de Guy, parce que #1095 est sur le
  chemin de la première facture (Q4) et que #1150 bloque un geste mensuel ; #1051 a un contournement simple (écrire
  « 1234 »), et sa probabilité est basse dans l'usage de Guy, qui écrit les milliers à l'apostrophe. Avec deux lignées
  (C-e360-4), l'ordre ne retarde aucune story de plus d'un rang.
- **Écartée** : l'ordre par impact strict (36-3 avant 36-2) : contredit l'ordre écrit par Guy sans gain mesurable.
- **Réversible** : oui ; l'orchestrateur peut permuter 36-2 et 36-3 dans la lignée A.

### C-e360-2 — Calendrier de la 0.17.8 : coupure au 2026-11-26

- **Contexte** : publier avant le 2026-11-30, mise à jour du NAS par Guy comprise.
- **Retenu** : au 2026-11-26, la 0.17.8 se publie avec les stories terminées ; 36-1 doit y être. Calque de la règle de
  l'Epic 35.
- **Écartée** : attendre les cinq stories : ferait peser la date de la première facture sur #931, la moins pressante.
- **Réversible** : c'est à Guy de déplacer la date.

### C-e360-3 — La règle d'arrêt vaut pour toute migration, en phase 1

- **Contexte** : le mandat dit « sans migration ; si une story en exige une, arrête-toi ». Le `CLAUDE.md` (P1, constaté
  à la 0.13.3) établit que toute migration empêche le binaire précédent de démarrer.
- **Retenu** : toute migration en phase 1 déclenche l'arrêt de l'orchestrateur ; une fiche qui en envisage une le dit
  avant de l'écrire. L'étape 0 n'en a trouvé aucune.
- **Écartée** : ne s'arrêter que sur une migration breaking (C-e350-5, même raison).
- **Réversible** : c'est à Guy de l'assouplir.

### C-e360-4 — Deux lignées, une par voie de base

- **Contexte** : deux serveurs MariaDB de dev (voies 1 et 2) ; `gate-ci.sh` ne connaît que la voie 1 ; une story qui
  touche `kesh-db` passe le gate complet même en boucle.
- **Retenu** : lignée A, voie 2 : 36-1 → 36-2 → 36-3 (frontend ; `AccountAutocomplete.svelte` partagé par 36-1 et, si
  H1, 36-2) ; lignée B, voie 1 : 36-4 → 36-5 (36-4 seule à toucher `kesh-db`). 36-3 peut se développer en parallèle de
  36-2 si 36-1 est fusionnée (aucun fichier commun établi entre 36-2 et 36-3).
- **Écartées** : (a) cinq stories en parallèle : deux voies seulement, et `AccountAutocomplete` partagé ; (b) tout en
  série : retarde la 0.17.8 sans gain.
- **Réversible** : oui ; l'orchestrateur réattribue les voies au prompt de chaque story.

### C-e360-5 — 36-2 : reproduire d'abord ; #1151 n'entre que si la cause l'exige

- **Contexte** : la cause de #1150 n'est pas établie (étape 0, H1 à H3). #1151 (fenêtre trop étroite, P4) touche le même
  dialogue.
- **Retenu** : la fiche commence par une reproduction (Vitest ou E2E) qui départage H1, H2 et H3, cite le test rouge, et
  borne le correctif à la cause établie. #1151 entre dans 36-2 **si** la reproduction montre que la largeur masque le
  message ou les valeurs (H3) ; sinon elle reste ouverte. Si la cause est H1, la fiche dit si la règle de 36-1 (le
  brouillon résolu avant tout envoi) se réemploie.
- **Écartées** : (a) écrire la fiche sur une hypothèse : la 35-7 a montré qu'un correctif du composant a des effets
  sur tous ses consommateurs ; (b) prendre #1151 d'office : P4, hors du chemin du défaut si la cause est H1 ou H2.
- **Réversible** : oui.

### C-e360-6 — 36-3 : le prédicat de la 35-6, la grammaire du journal ; périmètre borné à l'écriture manuelle

- **Contexte** : étape 0 de 36-3 ; la grammaire de la 35-6 et celle du journal diffèrent ; le même symptôme existe sur
  trois autres saisies.
- **Retenu** : extraire de `openingAmountKind` le **seul prédicat d'ambiguïté** vers un utilitaire partagé, que les deux
  écrans appellent (DRY) ; l'appliquer à l'écriture manuelle (et à l'assistant d'achat, même module
  `features/journal-entries`) ; un message propre au format du journal, qui ne conseille pas l'apostrophe tant qu'elle y
  est refusée. Les autres sites (soldes d'ouverture des réglages, prix unitaire des factures fournisseur) sont
  **inventoriés par le symptôme** dans la fiche : intégrés si le total reste à cinq modules au plus, sinon ouverts en
  issue, une par site. La quantité reste hors de la règle.
- **Amendé le 2026-11-14 sur décision de Guy** (réponse Q3 au kickoff : « ok ») : les autres sites **ne s'intègrent
  pas** — ils s'ouvrent en issue, une par site, dès le kickoff (soldes de départ des réglages : #1050, déjà ouverte ;
  prix unitaire des factures fournisseur : #1159, ouverte ce jour, où le contrôle du centime de #687 arrête « 1.234 »
  mais non « 1.250 ») ; et le journal **admet l'apostrophe** des milliers
  (« 1'234,50 »), comme le solde d'ouverture de la 35-6 — le message d'erreur actuel conseille une forme que le
  journal refuse. L'option (b) ci-dessous est donc retenue.
- **Écartées** : (a) changer `isValidAmount` pour tous ses appelants : touche le relevé manuel, les quantités et cinq
  écrans d'un coup, au-delà de la règle de splitting ; (b) ~~admettre l'apostrophe au journal~~ — retenue par Guy, voir
  ci-dessus ; (c) aligner le serveur : C-e350-19 a établi qu'un client API qui envoie « 1.234 » veut 1,234.
- **Réversible** : oui ; un refus en plus à l'écran, aucune donnée écrite.

### C-e360-7 — 36-4 : les deux annulations de règlement, et non l'enveloppe de test

- **Contexte** : étape 0 de 36-4 ; la clé est déjà portée par la route du dé-rapprochement ; elle se perd dans les deux
  `cancel_settlement_in_tx`.
- **Retenu** : `invoice_settlements_write::cancel_settlement_in_tx` et `supplier_invoices::cancel_settlement_in_tx`
  prennent un `Acteur` ; leurs enveloppes et les deux routes d'annulation de règlement le passent ; le dé-rapprochement
  passe le sien. La garde retire les entrées `AConvertir` correspondantes ; `api-external.md:404` est récrit ;
  l'enveloppe `reconciliation_cancel::cancel` (tests seuls) prend un `Acteur` ou est documentée comme telle. Ces sites
  sortent du périmètre des lots 36-11b et 36-11c.
- **Écartées** : (a) corriger seulement l'appel du dé-rapprochement : la route d'annulation d'un règlement garderait le
  défaut, alors que la fonction est la même ; (b) laisser #1131 au lot 36-11b de la phase 2 : Guy l'a placée en phase 1.
- **Réversible** : oui ; l'attribut `user_id` des entrées reste inchangé, seule `actor_api_key_id` s'ajoute.

### C-e360-8 — 36-5 : un correctif du produit, prouvé par répétition

- **Contexte** : la cause de KF-056 est écrite ; le défaut touche aussi un utilisateur réel (`/admin/restore`).
- **Retenu** : corriger le produit (le battement ne doit plus pouvoir renvoyer à la connexion pendant ou juste après un
  import qui invalide la session, et le compte rendu reste affiché jusqu'à ce que l'utilisateur le quitte ; forme exacte à
  la fiche) ; **preuve par répétition** : la spec rejouée N fois (`--repeat-each`, N fixé à la fiche, au moins 20) avant
  le correctif — au moins un rouge attendu, journal joint — et après — aucun rouge ; KF-056 retirée des échecs tolérés.
- **Écartées** : (a) corriger la spec seule (attendre le battement) : masque l'effet produit ; (b) ignorer tout 401 du
  battement en général : affaiblirait la fermeture de session (#876, #895).
- **Réversible** : oui.

### C-e360-9 — La 35-8b découpée en quatre lots par domaine

- **Contexte** : la 35-8b est `superseded` (Q3) ; sa validation P1 (`kesh-gate-logs/e35-8b-validation-p1.md`, V1-4)
  chiffre ≈ 140 fichiers et 1000 appels et suggère des lots (tiers, facturation, fournisseurs et paiements, réglages et
  exercices).
- **Retenu** : 36-11a tiers et catalogue ; 36-11b facturation client ; 36-11c fournisseurs et paiements ; 36-11d
  réglages, exercices, société, restauration (`RESTAURATION`, V1-1), démonstration (`pieges.rs`, #1132, V1-2), table
  finale de la garde et documents (V1-3), qui porte `closes #431`. Chaque fiche chiffre son rayon à l'étape 0 ; un lot qui
  dépasse cinq modules se redécoupe. Ordre : a, b, c en série (même garde), d en dernier. Après 36-4.
- **Écartées** : (a) une seule story : refusée par la P1 de la 35-8b ; (b) un lot par dépôt (une douzaine) : chaque lot
  touche la garde, douze fusions en série.
- **Réversible** : oui.

### C-e360-10 — Le plafond de la story de solde : huit issues, cinq modules au plus

- **Contexte** : règle du 2026-11-14 — une story de solde en fin d'epic, plafonnée par un nombre d'issues fixé au kickoff.
  À l'Epic 35, 69 issues de dette pour treize stories (≈ 5 par story), dont 51 P4 ; la règle « Tarir la source » doit
  en réduire le flux.
- **Retenu** : **huit issues au plus**, choisies parmi celles qu'ont ouvertes les stories 36-* (revues comprises), par
  impact puis par probabilité, dans **cinq modules au plus** — pour que la story reste dans la règle de splitting et
  qu'une revue P1 à trois lentilles relise chaque correctif. Placée en fin d'epic, avant le tag ; pas de story de solde
  pour la 0.17.8 (calendrier). **Confirmé par Guy le 2026-11-14** (Q1 et Q2 du kickoff : « ok »).
- **Écartées** : (a) un plafond proportionnel (la moitié des issues ouvertes) : non fixé au kickoff, contraire à la
  règle ; (b) douze issues ou plus : déborderait cinq modules, donc deux stories ; (c) quatre : laisserait le flux
  attendu (une à deux issues par story avec la règle nouvelle, quatorze stories et lots) presque intact.
- **Réversible** : oui ; Guy peut fixer un autre nombre avant la story.

### C-e360-11 — Le CHANGELOG ne s'écrit pas dans les branches de story

- **Contexte** : action E8 ; le CHANGELOG a demandé deux fois un outil hors consigne à l'intégration de la 0.17.7.
- **Retenu** : chaque fiche porte une section *Note de version* (rubrique, texte, issues) ; aucune branche de story ne
  touche `CHANGELOG.md` ; l'orchestrateur assemble `## [0.17.8] — Non publié` à l'intégration, **par l'outil Edit seul**,
  dans l'ordre des stories.
- **Écartée** : la règle d'union de l'Epic 35 (C-e350-8) : mécanique mais source des conflits de l'intégration.
- **Réversible** : oui.

### C-e360-12 — Clés de la phase 2, dans l'ordre de Guy

- **Contexte** : ordre donné par Guy le 2026-11-14 ; une clé `backlog` garde l'epic ouvert (définition du `CLAUDE.md`).
- **Retenu** : 36-6 #823, 36-7 #1125 (échéance 2026-12-28), 36-8 #1158 avec #268 (migration), 36-9 #1044, 36-10 #1147
  avec #1053, 36-11a à 36-11d (#431, #1132), 36-12 #944, 36-13 #235, 36-14 solde ; toutes `backlog`. **Aucune clé** pour
  #1097 (seulement si Guy le demande), ni pour #1151 à #1157, ni pour les chores E2/E3.
- **Écartée** : créer une clé pour #1097 : elle garderait l'epic ouvert pour un point conditionnel.
- **Réversible** : oui ; une clé s'ajoute ou passe à `superseded`.

### C-e360-13 — #1125 avant #1158 malgré l'ordre de date

- **Contexte** : Guy a placé #1125 en deuxième de la phase 2, « à terminer d'ici le 2026-12-28, de cette manière on peut
  travailler sur d'autres points avant ».
- **Retenu** : rang 7 (après #823), avec l'échéance écrite ; si #823 se découpe et menace la date, 36-7 passe devant —
  décision de l'orchestrateur, au plus tard le 2026-12-14.
- **Écartée** : #1125 en tête de la phase 2 : contredit l'ordre de Guy.
- **Réversible** : oui.

### C-e360-14 — README : rien à changer au kickoff

- **Contexte** : règle « synchroniser le planning du README à chaque commit ».
- **Retenu** : la *Feuille de route* du README ne porte qu'une ligne par **version publiée** (v0.17.5 à v0.17.7 :
  `README.md:241-243`), ajoutée par la PR de release ; aucun kickoff n'y a ajouté de ligne (`git log -- README.md` : les
  trois derniers commits sont des releases). La ligne de la 0.17.8 s'écrira à sa release.
- **Écartée** : une ligne « en cours » pour l'Epic 36 : le tableau n'en porte pour aucun epic.
- **Réversible** : oui.

### C-e360-15 — Le CR #1161 (« compte analytique ») entre en phase 2

- **Contexte** : CR ouvert le 2026-11-14 à la demande de Guy (#1161) ; l'orchestrateur a recommandé la phase 2 de l'Epic 36 ;
  Guy : « ok pour ta recommandation » (même jour).
- **Retenu** : clé `36-15-compte-analytique`, placée après 36-13 et **avant la story de solde 36-14** (qui reste la dernière
  avant le tag) ; vocabulaire visible seulement — écrans, messages des quatre langues, manuels, `docs/api-external.md` —, les
  identifiants internes `project*` (table, colonnes, route, champs de l'API) restent ; aucune migration. Les traductions se
  fixent à la fiche.
- **Écartées** : (a) renommer aussi les identifiants internes : migration breaking (`RENAME`) et changement de contrat de
  l'API, sans gain pour l'utilisateur ; (b) une epic suivante : les rapports de la première clôture porteraient l'ancien terme.
- **Réversible** : oui ; textes seulement.

### C-e360-16 — Le CR #1167 (réimputation analytique) entre en phase 2, après 36-15

- **Contexte** : CR ouvert le 2026-11-14 (#1167) : changer le compte analytique d'une facture même payée. Guy a préféré le geste
  dédié au retour en brouillon (qui exige de défaire règlements et rapprochements, et que refuse une facture envoyée par courriel
  ou d'un exercice clos), puis : « oui, place #1167 dans la phase 2 avec #1161 » (même jour).
- **Retenu** : clé `36-16-reimputation-analytique`, **après 36-15** (même objet ; le vocabulaire « compte analytique » est fixé
  d'abord) et avant la story de solde 36-14. Périmètre : celui du CR (facture client et fournisseur, tout statut ; écriture
  manuelle ; propagation aux écritures de vente ou d'achat, règlements, avoirs, solde ; audit ; exercice clos permis avec
  avertissement ; API externe, pas MCP). Sans migration attendue ; la fiche le confirme à l'étape 0, sinon elle le dit avant
  d'écrire (la phase 2 admet une migration, à sauvegarde exigée).
- **Écartées** : (a) le retour en brouillon : voir le CR ; (b) la répartition d'une facture entre plusieurs comptes analytiques,
  ligne par ligne : non demandée ; (c) une epic suivante : utile avant la première clôture.
- **Réversible** : oui ; le geste ne touche que `project_id`.

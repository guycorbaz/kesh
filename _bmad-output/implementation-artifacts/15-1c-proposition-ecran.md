# Story 15.1c : L'écran « Postes ouverts » — consulter, lettrer, délettrer

## Status

ready-for-dev *(réécrite le 2026-10-08, à valider — `bmad-create-story validate` avant tout développement)*

## Story

**As a** indépendant, PME ou fiduciaire,
**I want** un écran où je choisis un compte et une date, vois ce qui y reste ouvert et pourquoi,
lettre à la main ce que Kesh ne lettre pas seul — en m'appuyant sur ses propositions —, et
délettre une erreur,
**so that** je tienne mes comptes de tiers et de passage soldés sans quitter Kesh, et sache, à
la clôture, ce que porte chacun d'eux.

Dernière des quatre sous-stories du lettrage (#518). **Suppose 15-1a, 15-1a2 et 15-1b livrées** — la
15-1a étant découpée (C124) en **15-1a-i** (la marque, les routes) et **15-1a-ii** (les gardes), les
deux ; les renvois « 15-1a Rn / ACn » gardent leur numéro (table de `15-1a-socle-lettrage.md`).
Story **frontend + manuel + E2E** ; aucune route neuve.

## Reprise du 2026-10-08 — ce qui change

La fiche d'août portait le **moteur** de proposition et l'écran. Le moteur est passé en 15-1b
(backend), avec ses deux réserves tranchées ; ses décisions ouvertes au terme de la passe 3
sont closes : (3) **pas d'acceptation par lot** — chaque proposition s'accepte par un
`POST /api/v1/letterings` (15-1a), et une proposition périmée rend son refus nommé
(`LETTERING_LINE_ALREADY_LETTERED`, `LETTERING_UNBALANCED`…), si bien que le pattern
`FailedProposal` ne s'applique pas ; (4) **l'exposition du code** est faite par la 15-1a
(`JournalEntryLineResponse.letteringCode`) et la 15-1b (`LedgerLine.lettering_code`).

**Q4 du dégel — depuis quel écran ?** (C95) Un **écran dédié** `/open-items`, entrée de menu
**« Postes ouverts »** dans le groupe **Mensuel** (`+layout.svelte:107-111`, entre
« Réconciliation » et « Rapports ») ; atteint aussi depuis le **Grand livre** d'un compte
lettrable (lien « Postes ouverts de ce compte ») et depuis la **fiche d'écriture** (le code d'une
ligne lettrée ouvre le groupe). Écartés : un onglet du Grand livre (un rapport en lecture seule
n'est pas le lieu d'une action d'écriture) ; un écran « compte » (il n'en existe pas, `/accounts`
est le plan comptable).

## Reçu de la 15-1a — validation P2 du socle (2026-10-09)

*Section ajoutée par la remédiation de la validation P2 de la 15-1a (registre C113). Elle ne
réécrit pas cette fiche : elle liste ce que le socle a changé et que **cette** story doit intégrer
à sa propre validation.*

1. **Le refus des périodes change de code** : `LETTERING_FISCAL_YEARS_CLOSED` (cité à l'AC6 de cette
   fiche) est remplacé par **`LETTERING_ALL_LINES_IN_CLOSED_PERIODS`** — « Toutes ces lignes sont dans
   une période close — exercice clôturé, exercice suivi d'un exercice clôturé, ou période verrouillée :
   le lettrage n'y change plus. » Il vaut au lettrage comme au délettrage.
2. **La section du manuel sur le lettrage** énonce la règle entière : au moins une ligne « en période
   ouverte » (exercice ouvert, aucun exercice postérieur clos, date après le verrou de période). La
   15-1a écrit déjà la phrase symétrique au § du verrou de période (`user-manual.tex:578-583`).
3. **Le glossaire** (`user-manual.tex:2308`) est réécrit par la 15-1a avec une dernière phrase
   provisoire — « Dans cette version, le lettrage manuel et le délettrage se font par l'API
   (`/api/v1/letterings`) ; l'écran viendra. » — que **cette** story retire en livrant l'écran.
4. **`LETTERING_LINE_OWNED_BY_DOCUMENT`** porte un texte neutre (« Une de ces lignes appartient à une
   pièce : elle ne se lettre ni ne se délettre à la main. ») : il sert aussi au refus de dissoudre un
   groupe `reversal` qui contient une ligne de pièce.
5. **Prérequis** : la 15-12a ; ordre **15-12a → 15-12b → 15-1a → 15-1a2 → 15-1b → 15-1c** (C112).

*Ajouts de la remédiation de la validation P3 du socle (2026-10-09, registre C124, C126, C127) :*

6. **Ordre** : la 15-1a est découpée (C124) — **15-12a → 15-12b → 15-1a-i → 15-1a-ii → 15-1a2 → 15-1b →
   15-1c**.
7. **Le manuel de la modification est déjà touché** par la 15-1a-ii : un **sixième** point dans la
   liste des conditions de *Modifier* (`user-manual.tex:490-508` au 2026-10-09 — « aucune de ses lignes
   n'est lettrée »), et la réserve « lettrée » aux sites qui promettent « modifiable tant que l'exercice
   est ouvert ». L'AC12 de cette fiche (« `ENTRY_LETTERED` — si la 15-1a ne l'a pas déjà fait ») n'a
   donc qu'à **renvoyer** de ce point à la section du lettrage qu'elle écrit, et à retirer « le
   délettrage se fait par l'API dans cette version ».
8. **`ENTRY_LETTERED` parle en dernier** (C126) : quand l'écran l'affiche, l'écriture est en période
   ouverte et le délettrage qu'il prescrit aboutit — la section du manuel peut le dire sans réserve.
9. **Le glossaire** est désormais à `user-manual.tex:2323` (au 2026-10-09 ; `:2308` au point 3).
10. **CHANGELOG** : la 15-1a-i complète la section *Ajouté* existante et annonce sous *Modifié* les champs
    et colonnes neufs ; la 15-1a-ii réécrit les entrées #532 et annonce la contre-passation qui lettre.
    L'entrée « lettrage » que l'AC14 complète est celle de la 15-1a-i.
11. **Les lignes des routes de lettrage** portent `fiscalYearId` et `fiscalYearName` (C127) : un groupe à
    cheval peut montrer deux « écriture n° 12 » — l'écran affiche l'exercice avec le numéro.

## Critères d'acceptation

**AC1 — Choisir un compte et une date.** Sélecteur des comptes **lettrables** (15-1a R4 — même
règle que le serveur ; la liste vient du serveur, pas d'une règle recopiée en TypeScript), date
`asOf` (défaut : aujourd'hui), état dans l'URL (`/open-items?accountId=…&asOf=…`), pour que le
lien du Grand livre et un rechargement retombent sur la même vue.

**AC2 — La liste.** Colonnes : date, n° d'écriture (lien vers la fiche), journal, libellé, pièce
(lien vers la facture / l'avoir / la facture fournisseur / la transaction), débit, crédit, motif.
En pied : **total des postes ouverts** et **solde du compte à la date**, égaux (15-1b AC2) — et
une phrase qui le dit. Pagination du serveur.

**AC3 — Les motifs, en clair** (15-1b AC4), un libellé et une aide par motif :
`partiallySettled` (« facture partiellement réglée — reste dû : X »), `paidWithoutSettlementEntry`
(« marquée payée avant la v0.12.0, sans écriture d'encaissement : le compte porte encore cette
créance »), `letteredAfterAsOf` (« lettrée après cette date, par … »), `unlettered`.

**AC4 — Lettrer à la main.** Cases à cocher sur les lignes **lettrables à la main** (une ligne de
pièce n'a pas de case, et une infobulle dit pourquoi : « son lettrage suit ses règlements ») ;
**somme de la sélection** affichée en continu ; bouton **« Lettrer »** actif si ≥ 2 lignes et
somme **nulle** ; sinon le bouton dit ce qui manque (« la sélection ne s'équilibre pas : écart
X »). Un refus du serveur s'affiche **par son message** (codes de la 15-1a AC3), jamais en erreur
générique. Après succès : la liste se recharge, le code du groupe est annoncé.

⚠️ **Le lettrage à N lignes est permis** (règlement groupé manuel, acompte imputé sur plusieurs
factures… hors pièces) ; seules les **propositions** sont limitées aux paires (15-1b AC5).

**AC5 — Les propositions.** Panneau **« Rapprochements proposés »** (15-1b AC5), chaque paire avec
ses deux lignes et un bouton **« Lettrer »** ; **aucun** lettrage sans clic (règle du `CLAUDE.md` —
« propose, ne crée jamais »). Le refus 422 « trop de lignes » s'affiche tel quel, l'écran reste
utilisable pour le lettrage manuel.

**AC6 — Voir et défaire un groupe.** Un groupe s'ouvre par son code (lien depuis la fiche
d'écriture, le Grand livre, ou un champ « Code ») : ses lignes, son origine en clair
(« règlement de la facture F-… », « contre-passation », « manuel »), et **« Délettrer »** pour les
origines `manual` et `reversal`, si l'utilisateur est Comptable ou Admin. Groupe `document` : pas
de bouton, la phrase « ce lettrage suit les règlements de la pièce — annulez le règlement depuis
la facture ». Refus `LETTERING_FISCAL_YEARS_CLOSED` affiché par son message.

**AC7 — La frontière avec la réconciliation, pour l'UTILISATEUR** (D6 d'août, conservée) : un
bandeau visible dit que cet écran **solde des lignes de comptes de tiers et de passage entre
elles**, et que **le rapprochement des relevés bancaires** se fait dans *Mensuel →
Réconciliation* ; les comptes bancaires n'apparaissent pas au sélecteur (15-1a R4). Le test E2E
l'atteint par un `data-testid`, jamais par son libellé.

**AC8 — Écart avec la Balance**, dit à l'écran (15-1b, définitions) : « ce total est cumulatif ;
la Balance d'un exercice ne lit que ses écritures ».

**AC9 — Le code visible ailleurs.** Fiche d'écriture `/journal-entries/[id]` : colonne « Lettrage »
(code en lien vers le groupe) ; Grand livre (`GeneralLedgerView.svelte`) : même colonne, et le lien
« Postes ouverts de ce compte » si le compte est lettrable. Refus `ENTRY_LETTERED` sur
**Modifier/Supprimer** une écriture : message affiché, avec lien vers le groupe.

**AC10 — Rôles.** Consultation : voit tout, n'a ni cases, ni « Lettrer », ni « Délettrer ».

**AC11 — i18n.** Clés `open-items-*` (dossier `features/open-items/` — propriété vérifiée par
`lint-i18n-ownership`) et les quelques clés `journal-entries-*` / `reports-*` d'AC9, dans les
**quatre** locales ; `nav-open-items`. Les bornes **exactes** de
`frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`, etc., `:491-494`) sont relevées et
justifiées en commentaire.

**AC12 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`) : une section **« Lettrage et
postes ouverts »** — ce qu'est un lettrage (groupe à somme nulle, sur un compte), les trois
origines, qui lettre quoi (les pièces d'office, la contre-passation d'office, le reste à la main),
la date et l'invariant, les propositions, le délettrage et sa borne d'exercice (C94), **ce que le
lettrage ne fait pas** (pas de lettrage partiel, pas de tolérance de montant : le solde du reste
pour une facture, une écriture d'ajustement sinon ; une facture payée par écriture manuelle se
règle sur la facture), la frontière avec la réconciliation. Et la **propagation** dans les
sections existantes, **toutes greppées** : Modifier/supprimer une écriture (`:483`,
`ENTRY_LETTERED` — si la 15-1a ne l'a pas déjà fait), contre-passation (`:604`), règlements
(`:1122`), avoirs (`:1225`), factures fournisseurs (`:1354-1372`), Grand livre (`:1790`).
PDF régénéré (`make fr`), contrôlé **aplati** (`pdftotext … | tr '\n' ' '`).

**AC13 — E2E** (`frontend/tests/e2e/open-items.spec.ts`), sélecteurs `data-testid` seuls (garde
#326) : (1) une facture soldée par un règlement n'apparaît pas ouverte, sa créance porte un code ;
(2) deux écritures manuelles opposées sur un compte de passage → proposées → « Lettrer » → plus
ouvertes, code visible sur la fiche d'écriture ; (3) délettrer → de nouveau ouvertes ; (4) la
sélection déséquilibrée garde « Lettrer » inactif ; (5) le bandeau de frontière est présent.
**Lancée au dernier commit de code** (D7, rétrospective de l'Epic 25), jugée fichier par fichier
contre `docs/testing.md` § « Les échecs attendus ».

**AC14 — CHANGELOG** : l'entrée « lettrage » de `[0.13.0]` (15-1a, 15-1a2) est complétée par
l'écran — **une** entrée cohérente, pas trois fragments.

## Tasks

- [ ] **T1** (AC1–AC6, AC8, AC10) — `frontend/src/lib/features/open-items/` (API, types, composants)
      et la route `frontend/src/routes/(app)/open-items/+page.svelte` ; entrée de menu.
- [ ] **T2** (AC7) — Bandeau de frontière.
- [ ] **T3** (AC9) — Fiche d'écriture, Grand livre, refus `ENTRY_LETTERED`.
- [ ] **T4** (AC11) — i18n quatre locales, bornes du test des clés.
- [ ] **T5** (AC12) — Manuel FR + PDF ; DE/IT/EN : noter « à traduire » si le manuel y est vide.
- [ ] **T6** (AC13) — Spec E2E ; tests Vitest des composants (somme de sélection, état du bouton,
      cases absentes sur une ligne de pièce, rôle Consultation).
- [ ] **T7** (AC14) — CHANGELOG ; `README.md` « Feuille de route » (le lettrage livré) ;
      `website/` si une page le promet ou le tait.

## Dev Notes

- Modules : `features/open-items`, `features/journal-entries`, `features/reports`, `kesh-i18n`,
  `docs/manual` — cinq, au seuil du découpage préventif.
- Le sélecteur E2E ne se fige jamais sur un libellé traduit (garde #326, allowlist décroissante).
- Le montage E2E local : `KESH_COOKIE_SECURE=false`, `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE` (CLAUDE.md).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

### Reçu de la validation P3 du socle — 2026-10-09 (Opus 5.5, remédiation de la 15-1a)

Section « Reçu de la 15-1a » complétée des points 6 à 11 (registre C124, C126, C127) : ordre avec la
15-1a découpée ; le manuel de la modification (sixième condition, réserves) déjà touché par la
15-1a-ii ; `ENTRY_LETTERED` en dernier ; glossaire à `:2323` ; rubriques du CHANGELOG ; exercice par
ligne dans les routes. Dépendance de tête mise à jour. Corps non réécrit.

### Reprise du 2026-10-08 — réécriture contre le modèle réel (Opus 5.5, en autonomie)

Corps réécrit (registre C95, C99). Le moteur de proposition passe en 15-1b ; la fiche devient
l'écran seul, avec l'emplacement tranché (Q4 du dégel). Les décisions ouvertes (3) et (4) de la
passe 3 d'août sont closes (tête de fiche). **14 critères** (AC1–AC14), **7 tâches** (T1–T7),
recomptés depuis ce fichier.

*Entrées antérieures à la reprise — le corps qu'elles décrivent a été remplacé :*


### Passe 4 de `validate` — 2026-08-26 (Sonnet, contexte frais)

**1 HIGH, 2 MEDIUM, 1 LOW.** Sévérité décroissante (`2 HIGH` → `1 HIGH`).

⛔ **P4-1 (HIGH) — l'arbitrage affirmait « le même ensemble », et la décision ouverte (2) du
MÊME document le contredisait trois paragraphes plus loin.** Un écart structurel subsistait :
15-1b borne sa vue aux comptes `Receivable`/`Payable` **dans la requête** ; le moteur n'avait
**aucune** borne de rôle. On pouvait donc pointer le moteur sur le **compte bancaire ledger** —
que la vue n'ouvrira jamais — et y apparier des lignes que la réconciliation gère par un tout
autre mécanisme. 15-1c y répondait par **un bandeau** : **un bandeau ne protège pas l'API**.

→ **AC8** : le moteur ne travaille que sur les comptes que la vue ouvre, même borne, dans la
requête. ⚠️ Sur **`singleton_role`**, jamais sur `role` — leçon déjà payée par 15-1b, dont la
colonne générée vaut `NULL` pour un compte archivé.

✅ **L'alignement des deux fiches est désormais RÉEL**, et non plus seulement proclamé.

**P4-2 (MEDIUM) — résidu de propagation** : T2 prescrivait encore l'inscription au Pattern 5,
qu'AC6 venait de déclarer **non applicable** en passe 3. Corriger le critère sans greper la
tâche : le geste même que la § *Propagation post-patch* décrit, et la troisième fois qu'il se
produit dans cet epic.

**P4-4 (MEDIUM)** — **AC4-bis n'avait ni tâche ni test** depuis sa création en passe 3, alors
que son propre texte en exigeait un. Sans le tri, le plafond tronque un ensemble non ordonné
et évince **en premier** la contre-passation que la Réserve 2 protège — le critère introduit
pour fermer ce défaut serait resté non implémenté.

**P4-3 (LOW)** — le décompte de la passe 3 ne se recomptait pas : « quatre MEDIUM laissés » n'en
nommait que **trois**, et P3-9/P3-10 relèvent des LOW. Rectifié.

**Réfuté** : la borne de performance ne se dégrade **pas** avec l'alignement — l'absence de
prédicat réducteur était acquise avant l'arbitrage ; l'index composite est bien livré par le
socle ; et le paragraphe « ce que l'arbitrage supprime » décrit des findings de **15-1b**, ce
qui est défendable pour un texte partagé entre fiches sœurs.

La spec passe de **9 à 10 critères**. **Verdict : passe 5 due** — un HIGH au rapport, corrigé
ici.

### Arbitrage du Project Lead — 2026-08-26 : « ouvert » = « non lettré »

⛔ **La CINQUIÈME décision — celle que ni 15-1b ni 15-1c ne portait — est tranchée.** Les deux
passes 3, indépendantes, avaient conclu qu'elle décidait de la forme de la requête que les
deux stories allaient écrire.

> **« Ouvert » signifie « non lettré ». La vue ne regarde ni `paid_at`, ni aucun statut de
> facture, et ne joint aucune table de factures.**

✅ **Ce que l'arbitrage achète — un INVARIANT, pas une commodité.** Toute paire lettrée se
nettant exactement à zéro (15-1a AC4 et AC12), **la somme algébrique des lignes ouvertes d'un
compte égale son solde**, sans exception. C'est **testable en une assertion**, et c'est ce qui
rend enfin vraie la promesse du *so that* : *« justifier le solde d'un compte »*. Aucune des
deux définitions concurrentes ne le permettait.

⛔ **Ce qu'il coûte, et qui doit être assumé à l'écran** : une facture réglée par virement
importé réapparaît **ouverte** tant qu'elle n'est pas lettrée. C'est **comptablement vrai** —
la réconciliation ne crée aucune écriture, le compte porte toujours son débit — mais
contre-intuitif. **AC4 de 15-1b devient de ce fait le critère le plus important de la fiche**,
et il doit offrir un chemin vers le lettrage, pas seulement une explication.

✅ **Ce qu'il SUPPRIME, et c'est le plus notable** : **trois HIGH et trois MEDIUM des passes 1
à 3 tombent avec lui** — la contradiction entre fiches sœurs, la déclinaison en trois puis
quatre cas, les deux tables de factures, les deux écritures fournisseur, la troisième écriture
d'annulation non référencée. **Ce n'est pas une simplification cosmétique : c'est la
disparition de la classe entière de défauts que ces passes trouvaient**, tous nés de ce que la
vue tentait de concilier deux mécanismes que rien n'oblige à concilier.

⚠️ **Ce qu'il NE tranche PAS.** La Décision 1 (relance) reste ouverte et son enjeu se
**déplace** : la vue ne lit plus `paid_at`, mais les **cinq lecteurs** recensés continuent de
le lire — et le plus grave est comptable, pas cosmétique. `reconciliation.rs` proposera une
facture lettrée mais non marquée payée à un **second règlement** : **soldée deux fois**, une
fois en caisse et une fois en banque. La Décision 3 reste ouverte pour la même raison.

### Passe 3 de `validate` — 2026-08-25 (Opus, contexte frais)

⛔ **2 HIGH, 6 MEDIUM, 4 LOW. LA SÉVÉRITÉ REMONTE** — `2 HIGH+2 MED` → `2 MED+1 LOW` →
`2 HIGH+6 MED`. **Le critère de non-convergence de la § *Règle de splitting préventif* est
déclenché**, pour la seconde fois dans cet epic.

⚠️ **Mais le diagnostic n'est PAS « la fiche est trop large » — elle est trop COUPLÉE à ses
sœurs.** Cinq des huit findings > LOW sont des **coutures** entre 15-1a, 15-1b et 15-1c, et
deux n'existent que depuis la passe 2 de **15-1b**, tombée pendant cette revue. **Une passe 4
sur 15-1c seule ne verrait pas la suivante.** Ce qu'il faut n'est pas une passe de plus, mais
**une relecture des trois fiches ENSEMBLE sur la seule question « qu'est-ce qui est ouvert, et
qui le dit ».**

⛔ **P3-1 (HIGH) — 15-1c et 15-1b prescrivent, chacune par un test nommé, deux comportements
INCOMPATIBLES.** Sur le compte fournisseur, AC2 exige que la paire soit **proposée** ;
AC3-bis de 15-1b exige que le compte n'affiche **aucune ligne ouverte**. Le même écran dirait
« 0 ligne ouverte » et « 1 rapprochement proposé » sur **les mêmes deux lignes**. → **décision
ouverte**, c'est un arbitrage de produit.

⛔ **P3-2 (HIGH) — AC7 ne bornait pas ce qu'il disait borner.** Ses deux bornes agissaient
**après** l'appariement ; le `LIMIT 50` qu'il citait en modèle borne le **jeu candidat**, dans
un `WHERE` que la fenêtre **et** la tolérance réduisent. Or la passe 2 a retiré la fenêtre du
filtre et AC2 interdit `status`/`paid_at` : **il ne restait aucun prédicat réducteur**. Et
l'analogie était fausse — la réconciliation est **1 → N**, le lettrage **N → N**. → borne sur
le **jeu candidat**, plafond **chiffré à 500**, et la remarque que l'égalité stricte rend
l'appariement **groupable par montant**, donc linéaire.

**Trois MEDIUM corrigés ici** : **P3-3** le « classement » tranché en passe 2 n'existait dans
aucun critère — le plafond tronquait un ensemble non ordonné, évinçant **en premier** la
contre-passation que la Réserve 2 protège → **AC4-bis** ; **P3-4** la fenêtre figurait
**toujours** comme critère d'éligibilité dans le tableau, la passe 2 ayant corrigé les deux
sites qui en *parlent* et laissé les deux qui la *prescrivent* — le geste même que la
§ *Propagation post-patch* codifie ; **P3-6** ⚠️ **mon affirmation « ce document exige que tout
nouvel endpoint y figure » était FAUSSE** — le Pattern 5 impose un ordre à ceux qui prennent
**plus d'un verrou**, et les routes de 15-1c sont des routes de **lecture** ; **P3-9/P3-10** le
décompte d'index (trois, pas deux) et le titre du tableau, que sa propre colonne contredisait.

**Trois MEDIUM laissés en décisions ouvertes** *(rectifié en passe 4 : « quatre » n'en nommait que trois, et P3-9/P3-10 sont deux des quatre LOW, pas des MEDIUM corrigés)* — voir le § dédié : la borne de rôle du moteur,
le contrat d'acceptation et le pattern de lot, AC4 sans tâche ni test.

**Réfuté** : le déplacement de l'index vers 15-1a est **correct des deux côtés**, `CREATE INDEX`
n'impose aucun bump `min_required`, et la colonne « provenance » est exacte sur ses trois lignes
NEUF.

**Verdict : relecture croisée des trois fiches due, pas une passe 4 sur celle-ci.**

### Passe 2 de `validate` — 2026-08-25 (Haiku, contexte frais)

**0 HIGH, 2 MEDIUM, 1 LOW.** Sévérité décroissante (`HIGH → MEDIUM`) : convergence monotone.

**Aucune régression des patches de la passe 1** : AC6, AC7, la colonne « provenance » et le
critère `lettering_id IS NULL` sont vérifiés exacts au sol et jugés bien posés.

⚠️ **Les deux MEDIUM ne sont pas des défauts de raisonnement mais des ambiguïtés de
COORDINATION entre fiches** — la classe d'erreur que le split fabrique, et la troisième fois
qu'elle se manifeste dans cet epic.

⛔ **P2-1 — AC7 exigeait un index composite que PERSONNE ne créait.** 15-1a ne posait qu'un
`idx_jel_lettering (lettering_id)` simple ; le composite `(account_id, lettering_id)` n'était
la tâche d'aucune des deux fiches. → **Tranché : la migration vit dans le socle**, donc
`idx_jel_account_lettering` a été **ajouté au DDL de 15-1a** ; 15-1c le **consomme**.

⚠️ **Et les deux index sont nécessaires**, le préfixe gauche ne permettant pas de les
confondre : `(lettering_id)` sert « retrouver la contrepartie d'une marque »,
`(account_id, lettering_id)` sert « les lignes ouvertes du compte A » — la requête du moteur
**et** celle de la vue de 15-1b.

⛔ **P2-2 — « tranchée par défaut » ne disait pas LAQUELLE des deux conduites était le défaut,
et AC2 en dépendait.** → **Tranché : la fenêtre s'applique au CLASSEMENT, pas au filtre.**
C'est la seule lecture cohérente avec AC2, qui exige un test datant les deux pièces à **plus de
30 jours d'écart** : si la fenêtre filtrait, ce test **échouerait par construction**, et deux
développeurs auraient raison en même temps — celui qui écrit le moteur avec un filtre, celui
qui écrit le test tel qu'AC2 le prescrit. **C'est la contradiction « proposer ce qu'on refuse »
de la Réserve 1, transposée à la date.**

**P2-3 (LOW)** : « inscrire au tableau du Pattern 5 » ne disait ni quoi ni sous quelle forme —
le format des lignes voisines est désormais nommé.

**Vérifié et réfuté** : multi-devise et arrondis `DECIMAL(19,4)` sans objet ; AC1 et AC2
complémentaires, pas antagonistes ; la Réserve 1 correctement posée, sa conséquence énoncée.

**Verdict : passe 3 due** — deux MEDIUM au rapport. ⚠️ Aucun ne relève d'un arbitrage produit :
les deux sont tranchés ici sur la cohérence interne, et **restent réversibles d'un mot**.

### Passe 1 de `validate` — 2026-08-25 (Sonnet, contexte frais)

**2 HIGH, 2 MEDIUM.** Tous vérifiés au sol par l'orchestrateur avant application.

⚠️ **Les quatre findings répètent le motif des huit passes de 15-1a** : une lecture de la spec
contre elle-même les aurait tous laissés passer. **Seule la lecture des chemins de code de
l'Epic 8 et du schéma de `journal_entry_lines` les révèle.**

| | défaut | gravité | remède |
|---|---|---|---|
| **P1-2** | **Aucune mention de scoping multi-tenant** — zéro occurrence de `company_id`, « multi-tenant » ou « IDOR ». Or 15-1c ouvre une **surface d'API neuve** par un chemin **distinct** des routes que 15-1a a scopées, et `journal_entry_lines` n'a **aucun** `company_id` | **HIGH** | **AC6** — jointure, 404 indiscernable, inscription au Pattern 5 |
| **P1-4** | **Aucune borne de performance ni anti-DoS** — T1 et T2 tenaient en une phrase. La réconciliation en porte **trois** pour le même calcul : `LIMIT 50`, `MAX_PROPOSALS_LIMIT = 500` (*« défense anti-DoS contre `?limit=999999` »*) et un index dédié | **HIGH** | **AC7** — portée par compte, plafond serveur, composite `(account_id, lettering_id)` |
| **P1-1** | **La provenance de deux critères sur quatre était FAUSSE** : « même compte » et « sens opposés » **n'existent nulle part** dans l'Epic 8, dont le scoring est `0,50 montant + 0,40 référence + 0,10 contact`. Pire, `rules.rs` a **délibérément retiré** le seul filtre de sens qui ait existé | MEDIUM | colonne « provenance » au tableau ; T1 dit ce qui s'extrait et ce qui est neuf |
| **P1-3** | **Le critère `lettering_id IS NULL` manquait** au tableau **et** aux tests. AC10 de 15-1a protège la route, mais **rien n'empêchait le MOTEUR de re-proposer** une paire déjà lettrée | MEDIUM | critère ajouté, test nommé |

⛔ **P1-4 est aggravé par deux traits propres à cette story**, et c'est ce qui le rend HIGH
plutôt que MEDIUM : la **Réserve 2** envisage de retirer la fenêtre de dates du filtre — donc
de comparer toutes les lignes d'un compte sans borne temporelle — et **AC2 interdit de filtrer
sur le statut de facture**, si bien que l'ensemble candidat ne peut plus être réduit comme le
fait la réconciliation. Sur un compte fournisseur actif depuis plusieurs exercices — que **D3
autorise explicitement** —, l'appariement devient **quadratique et non plafonné**.

⚠️ **P1-3 dit quelque chose du découpage** : le socle protège la **route**, la story de l'écran
alimente le **moteur**, et le critère qui les relie n'était écrit ni dans l'une ni dans l'autre.
C'est la même classe de trou que l'égalité des montants, tombée entre 15-1a et 15-1c et
rapatriée en passe 3.

**Six pistes réfutées au sol**, dont : la tolérance de 5 centimes n'est **pas** justifiée par
les frais bancaires *dans le code* — ce narratif vient de la story mère, le commentaire réel
dit seulement *« réduit le candidate set sans accepter le mismatch »* ; le montant TTC/HT ne
pose **pas** ici le problème qu'il a posé à la réconciliation (#246), les lignes de grand livre
portant déjà le TTC ; la paire facture/avoir **est** structurellement exacte, l'avoir créditant
`total_ht + total_vat` au même compte ; et le règlement fournisseur crée bien une écriture au
même compte, sens opposé, même TTC, **pour les deux modes de règlement**.

La spec passe de **5 à 7 critères**. **Verdict : passe 2 due.**

### Création par split de la 15-1 — 2026-08-25

Issue du **split de la Story 15-1**. Recueille la correction majeure de la passe 1 sur D5
(les filtres de facture qui excluaient trois cas sur quatre) et le MEDIUM **P3-7** de la
passe 3 (la fenêtre de 30 jours qui tue la contre-passation).

⛔ **Deux réserves restent ouvertes** : la tolérance de montant face aux frais bancaires, et
la fenêtre de dates. Toutes deux sont tranchées **par défaut** dans la spec, avec leur
conduite alternative nommée — elles se changent d'un mot tant que le développement n'a pas
commencé.

# Epic 29 — registre des choix autonomes

*Registre tenu par union (jamais par écrasement). Chaque choix : contexte, option retenue, alternatives, réversibilité.
Fichier d'epic : `planning-artifacts/epic-29-preparer-cloture.md`. Consignes : `docs/agents-orchestration.md`.*

---

## Story 29-0 — Kickoff (doc seulement)

Choix faits au kickoff (2026-10-14), en autonomie, sur la rétrospective de l'Epic 28.

### C-29-0-1 — Deux releases : v0.13.4 pour la réconciliation, v0.14.0 pour le bouclement

- **Contexte** : Guy éprouve la réconciliation chaque mois avec ses relevés, mais ne testera le bouclement qu'en
  début 2027.
- **Retenu** : publier les trois correctifs de réconciliation (29-2 à 29-4) en **v0.13.4** dès qu'ils sont mergés,
  puis le bouclement (29-1) en **v0.14.0**. Les correctifs arrivent au NAS sans attendre une fonctionnalité que
  personne n'essaiera avant 2027.
- **Écartée** : une seule release en fin d'epic (les correctifs attendraient la 29-1, la plus longue).
- **Réversible** : oui (deux tags au lieu d'un, ou l'inverse).

### C-29-0-2 — « autonomie complète pour 28 » lu comme l'Epic 29

- **Contexte** : réponse de Guy du 2026-10-14 à la question « la même autonomie vaut-elle pour l'Epic 29 ? ».
  L'Epic 28 était clos à cette date.
- **Retenu** : autonomie complète jusqu'à l'image Docker pour l'Epic 29 ; la lecture a été annoncée à Guy, avec
  l'offre de revenir au travail local s'il l'entendait autrement.
- **Réversible** : oui, sur un mot de Guy.

### C-29-0-3 — La 29-1 commence par un inventaire de l'existant

- **Contexte** : la recommandation de l'Epic 29 reposait sur une prémisse en partie fausse (rétrospective 28, § 9) :
  la clôture et le report à nouveau existent déjà. Le texte de #232 est antérieur à ces livraisons.
- **Retenu** : la fiche de la 29-1 établit au code ce qui manque avant toute conception, puis découpe.
- **Écartée** : spécifier d'après le texte de #232 (« un kickoff écrit d'après l'issue répète ce que l'issue croit »,
  rétrospective 27, § 4.1).
- **Réversible** : oui.

---

## Story 29-2 — Un paiement reçu après l'échéance, ou après un rappel, est proposé ([#529])

### C-29-2-1 — Deux prémisses périmées, corrigées dans la fiche

- **Contexte** : #529 (2026-10-08) décrit une fenêtre « centrée sur la date de la facture » ; la consigne dit que le
  manuel « promet » que le rapprochement propose la facture.
- **Retenu** : établi au code sur `16eeed21` — depuis la 28-7, la fenêtre est `D − 30 ≤ F ≤ D + 1` (une seule
  définition, deux lecteurs) ; le défaut subsiste dans sa borne basse, ancrée sur la date de la facture. Le manuel
  (`user-manual.tex:1305`, `:2029`, `:2902`) dit déjà que le paiement tardif n'est pas proposé et renvoie au règlement
  depuis la fiche. Il change parce que la règle change, non parce qu'il mentait.
- **Écartée** : spécifier d'après le texte de l'issue.
- **Réversible** : sans objet (constat).

### C-29-2-2 — Voie 1 seule : la borne basse s'ancre sur max(date, échéance, dernier rappel)

- **Contexte** : #529 propose deux voies, « et/ou » : une fenêtre qui tient compte de l'échéance et des rappels, ou
  le choix manuel d'une facture ouverte à l'écran. L'orchestrateur exige une seule borne pour la proposition et
  l'acceptation (#548 non rouvert).
- **Retenu** : `A = max(F, E, R)`, la facture est candidate et acceptée si `F ≤ D + 1` et `A ≥ D − 30`. Ferme le cas
  courant (30 jours net payé en retard, paiement après rappel), garde une seule borne, aucun écran ne change.
- **Écartées** : (a) le choix manuel à l'écran — deux bornes ou une voie d'acceptation sans borne, plus un dialogue
  neuf, i18n, E2E, captures : hors du périmètre minimal ; (b) supprimer la borne basse — à score égal, la plus
  ancienne facture impayée de même montant passe devant (départage `invoice_id ASC`) et est la seule affichée et
  acceptée en lot : faux rattachement muet ; (c) l'échéance seule, sans les rappels — laisse ouvert le cas « après un
  rappel » que l'issue et le manuel nomment.
- **Réversible** : oui (une expression SQL et un paramètre de `check`).

### C-29-2-3 — Les rappels annulés ne comptent pas ; date du rappel = `DATE(sent_at)`

- **Contexte** : `invoice_reminders.cancelled_at` est une annulation soft (Admin, envoi accidentel), déjà exclue du
  niveau courant ; `sent_at` est un `DATETIME(6)`, UTC pour un e-mail, saisi pour un rappel manuel.
- **Retenu** : `MAX(sent_at)` des rappels `cancelled_at IS NULL`, ramené à sa date. L'écart de fuseau (un jour au
  plus) est un angle mort écrit, sans effet mesurable sur une borne de 30 jours.
- **Écartées** : compter les rappels annulés (un envoi accidentel élargirait la fenêtre) ; convertir le fuseau (aucun
  fuseau de société n'est modélisé pour cette colonne).
- **Réversible** : oui.

### C-29-2-4 — Une seule expression SQL de l'ancre, lue en forme jointe et en forme scalaire

- **Contexte** : la proposition filtre en SQL (avant `LIMIT 50`), l'acceptation tranche en Rust sur une facture.
  Deux écritures de la même règle divergent (précédent du reste dû, #416).
- **Retenu** : `INVOICE_PAYMENT_ANCHOR_SQL` + `invoice_payment_anchor_joins()` ; la proposition les met dans son
  `WHERE`, l'acceptation les lit par `invoice_payment_anchor_date` (même constante, `WHERE i.id = ?`). La comparaison
  à `D` reste `InvoiceDateWindow::check(F, A)` ; la parité est tenue par un test HTTP.
- **Écartées** : calculer `A` en Rust côté acceptation depuis `invoice.due_date` et une requête du dernier rappel (une
  seconde définition) ; filtrer en Rust après la requête (le `LIMIT 50` couperait avant le filtre).
- **Réversible** : oui.

### C-29-2-5 — Contrat d'API inchangé ; l'élargissement s'écrit à `api-external.md`

- **Contexte** : `RECONCILIATION_INVOICE_NOT_ELIGIBLE` / `payment_date_outside_window` / `window_days = 30` sont des
  valeurs que lit une intégration ; une acceptation refusée hier peut aboutir demain.
- **Retenu** : mêmes codes, raisons et `details` ; une phrase *(depuis la v0.13.4)* dans
  `docs/api-external.md` § *Accepter des propositions de rapprochement*, et la mention au CHANGELOG.
- **Écartée** : ajouter `details.anchorDate` (contrat élargi sans demandeur).
- **Réversible** : oui.

### C-29-2-6 — Ni frontend, ni i18n, ni capture, ni spec Playwright neuve

- **Contexte** : aucun texte ni contrôle de l'écran ne change ; le libellé de `payment_date_outside_window` est le
  repli générique, sans durée.
- **Retenu** : la frontière HTTP est exercée par des tests `spawn_app` (parité, cas de #529 bout à bout) ; la suite
  E2E complète tourne au dernier commit de code (précédent C-28-7-8, C-28-7-9).
- **Écartée** : une spec Playwright d'un paiement tardif (montage d'échéance et de rappel en E2E pour une assertion
  que le test HTTP fait déjà).
- **Réversible** : oui.

### C-29-2-7 — Ordre, `LIMIT 50` et départage inchangés ici ; leur coût accru est porté par #668 (Story 29-5)

- **Contexte** : l'ancre fait entrer parmi les candidates des factures échues, relancées, **ou pas encore échues** —
  dont des factures anciennes à échéance lointaine, candidates sans limite de temps (C-29-2-11) ; à score égal,
  `propose_matches` place la plus ancienne (`invoice_id ASC`, `matching.rs:104`) en tête, seule affichée. En amont,
  `ORDER BY i.date ASC, i.id ASC LIMIT 50` (`repositories/reconciliation.rs:258-259`) garde les 50 plus anciennes de
  même reste dû : des factures anciennes non échues peuvent en pousser dehors une facture échue.
- **Retenu** : ne rien changer à l'ordre, à la troncature ni au départage ici. Le départage à score égal et la
  troncature qui le précède sont portés par [#668] (Story 29-5, même release v0.13.4, mergée avant le tag —
  C-29-2-13), dont l'attendu, rectifié au regard de l'art. 87 CO, est : à score égal, les factures échues d'abord,
  puis l'ancienneté ; cas de test : deux factures de même reste dû, l'une ancienne à échéance lointaine (non échue),
  l'autre échue — l'échue en tête. L'ordre règle contre facture reste à la 29-3 (#527).
  *(Complété le 2026-10-14, validation P1, F3 : la première rédaction ne nommait que les factures « échues et
  relancées ».)* *(Rectifié le 2026-10-14, validation P2, R1/F1 et F5 — décision de l'orchestrateur, C-29-2-13 : la
  rédaction de P1 demandait de commenter #527 avec un départage par proximité de date, « la récente en tête » ; la
  29-3 l'a refusé (C-29-3-12) et ce critère irait contre l'art. 87 CO. Le coût de la troncature `LIMIT 50`, annoncé
  au titre sans être écrit, est ajouté.)*
- **Écartée** : changer le départage dans la 29-2 (empiète sur `kesh-reconciliation`, quatrième module ; porté par
  la 29-5).
- **Réversible** : oui.
- *(Annotation du 2026-10-14, revue de code P1, finding E6 (LOW) : aucun test ne fait traverser la coupure du
  `LIMIT 50` par une facture de la fenêtre élargie (`F` ancien, ancre récente), qui passe devant les récentes par le
  tri `(i.date, i.id)`. **Renvoyé à la 29-5** (#668), qui teste la troncature : un test écrit ici figerait l'ordre
  que la 29-5 doit remplacer — « les factures échues d'abord » —, et serait réécrit par elle. Aucun test ajouté.)*

### C-29-2-8 — Le contournement du reliquat est complété au manuel : rejeter la transaction

- **Contexte** : un règlement par virement saisi depuis la fiche débite le compte de la banque
  (`invoice_settlements_write.rs:130-154`) ; la transaction bancaire reste en attente et pourrait être passée une
  seconde fois. Le manuel n'en dit rien.
- **Retenu** : ajout mécanique aux paragraphes du manuel réécrits par la story : enregistrer depuis la fiche, **puis
  rejeter** la transaction.
- **Écartée** : ouvrir une issue séparée (la phrase vit dans les paragraphes que la story réécrit).
- **Réversible** : oui.
- *(Annotation du 2026-10-14, développement — information de l'orchestrateur : le geste prescrit est aujourd'hui
  **impraticable à l'écran** — une ligne « Aucune correspondance » n'a pas de case à cocher
  (`ReconciliationProposals.svelte`, l. 318-330), et *Rejeter* n'agit que sur les lignes cochées. **Décision de
  l'orchestrateur** : la **Story 29-6** (#665), même release v0.13.4, rendra possible le rejet d'une ligne sans
  proposition, et **doit être mergée avant le tag v0.13.4**. La 29-2 ne change pas son code pour cela et garde le texte
  prescrit du manuel.)*
- *(Annotation du 2026-10-14, revue de code P1, findings B1 = E2 = A1 (MEDIUM) — **décision de l'orchestrateur** :
  le texte du manuel et du CHANGELOG reste ; la **Story 29-8** (#670) le remplacera par « puis rattachez », la 29-6
  rendant une ligne sans proposition rejetable. La dépendance de publication, jusqu'ici écrite au seul registre,
  devient un **contrôle écrit** au fichier d'epic (`epic-29-preparer-cloture.md`, § *Releases*, « Liste de
  pré-release de la v0.13.4 ») : 29-6 et 29-8 mergées, et `grep -nF 'rejetez la transaction'
  docs/manual/fr/user-manual.tex` (plus une variante `grep -nE` qui attrape « en rejetant ensuite » et le CHANGELOG)
  ne rend plus le conseil sans le rattachement. Aucune ligne de code pour ce point.)*

### C-29-2-9 — Le reliquat (plus de 30 jours après l'échéance, sans rappel) : couvert par #666

- **Contexte** : après la story, un paiement reçu plus de 30 jours après l'échéance et sans rappel depuis 30 jours
  n'est toujours ni proposé ni acceptable à la réconciliation ; la voie 2 de #529 (choisir une facture ouverte à
  l'écran) le couvrirait.
- **Retenu** : reliquat écrit au manuel avec son contournement complet (C-29-2-8) ; il est couvert par
  [#666] (« Choisir à la main une facture ouverte pour une transaction bancaire créditrice », `enhancement`, ouverte,
  « Refs #529 »), pour qu'aucune limite ne reste sans issue (leçon § 4.3 de la rétrospective 28). *(Mis à jour le
  2026-10-14, validation P1, R6/F6 : la première rédaction demandait l'ouverture de l'issue, faite depuis.)*
- **Écartée** : l'intégrer ici (C-29-2-2).
- **Réversible** : oui.

### C-29-2-10 — `CAST(… AS DATE)` autour de l'ancre ; `DATE()` retiré déclaré mutation équivalente

- **Contexte** : validation P1, F4. Aucun `GREATEST` sur des dates n'existe au dépôt ; le type rendu (et son décodage
  `NaiveDate` par sqlx) n'aurait été établi qu'à la première exécution.
- **Retenu** : `INVOICE_PAYMENT_ANCHOR_SQL` = `CAST(GREATEST(…) AS DATE)` — le type devient une garantie. Retirer
  `DATE()` autour de `rmd.last_sent_at` est alors une mutation **équivalente** (M12), déclarée comme telle ; retirer
  le `CAST` (M13) est équivalente ou tuée selon ce que MariaDB rend, constaté au Dev Agent Record.
- **Écartée** : se fier au comportement constaté de `GREATEST` sans `CAST` (une constatation, non une garantie).
- **Réversible** : oui.

### C-29-2-11 — Le faux rattachement par une facture non échue ou à échéance lointaine : angle mort écrit

- **Contexte** : validation P1, F3. La justification d'origine (« l'ancre borne le risque à une facture vivante,
  échue ou relancée depuis moins de 30 jours ») était fausse pour les factures **non échues** : `A = E` grand, la
  facture reste candidate de `F − 1` à `E + 30` ; `due_date` n'a pas de borne haute (`routes/invoices.rs:765` ne
  garde que `due_date < date`), une échéance à un an ou une coquille à 2099 la rend candidate sans limite. À score
  égal, la plus ancienne passe devant (`matching.rs:97-106`, départage `:104` ; `candidates[0]` seul affiché).
- **Retenu** : la phrase de la fiche est réécrite (« échues depuis moins de 30 jours, relancées depuis moins de 30
  jours, ou pas encore échues ; ce dernier ensemble n'est pas borné dans le temps ») ; le risque est un **angle mort
  écrit**, inhérent à toute prise en compte de l'échéance, **porté par [#668]** (`bug`, P2) et traité par la Story
  29-5, même release v0.13.4, mergée avant le tag (C-29-2-13). L'aggravation vaut aussi pour le cas ordinaire : à
  30 jours net, les candidates passent de `F ≥ D − 30` à `F ≥ D − 60`, et deux mensualités impayées d'un même client,
  de même reste dû, deviennent candidates ensemble. *(Rectifié le 2026-10-14, validation P2, R1/F1 : la rédaction
  de P1 disait « que la 29-3 (#527) traite par le départage » ; la 29-3 le refuse (C-29-3-12), et aucune issue n'était
  citée.)*
- **Écartées** : (a) borner l'ancre (`LEAST(E, F + n)`) — une règle de plus, arbitraire, qui recréerait le reliquat
  pour les conditions longues ; (b) traiter le départage ici — empiète sur `kesh-reconciliation` (porté par la
  29-5) ; (c) une garde de saisie d'échéance
  maximale — hors de #529, sans demandeur.
- **Réversible** : oui.

### C-29-2-12 — « Dernier rappel non annulé » : site jumeau cité, non factorisé

- **Contexte** : validation P1, R7. `dunning_eligibility.rs:80` écrit déjà `MAX(sent_at) … WHERE company_id = ? AND
  cancelled_at IS NULL GROUP BY invoice_id`.
- **Retenu** : citer ce site au doc-comment de la jointure neuve et à l'inventaire (b), pour que la notion
  d'annulation change aux deux endroits ensemble.
- **Écartée** : factoriser — les agrégats diffèrent (`MAX(level_number)` en plus), et la dérivée de l'éligibilité
  prend un `company_id` lié là où les jointures du reste dû, patron suivi ici, n'en prennent aucun.
- **Réversible** : oui.

### C-29-2-13 — Le départage à score égal est porté par #668, Story 29-5, mergée avant le tag v0.13.4 (décision de l'orchestrateur)

- **Contexte** : validation P2, R1 = F1. La fiche et le registre confiaient l'angle mort du faux rattachement
  (C-29-2-11) à la 29-3 (#527), qui le refuse (C-29-3-12) ; l'issue [#668] le décrit, sans être citée. La 29-2
  **aggrave** le risque (F1 : candidates de `F ≥ D − 30` à `F ≥ D − 60` à 30 jours net).
- **Retenu** — **décision de l'orchestrateur, consignée comme telle, non un choix autonome de la fiche** : le départage
  à score égal est porté par [#668] (`bug`, P2), dont l'attendu a été **rectifié** en commentaire au regard de
  l'art. 87 CO (paiement sans désignation : dette exigible d'abord, puis la plus ancienne ; à score égal, factures
  échues d'abord, puis ancienneté). #668 devient la **Story 29-5**, qui sort dans la **même release v0.13.4** que la
  29-2 : **la 29-5 doit être mergée avant le tag v0.13.4**, si bien qu'aucune release ne livre l'aggravation sans son
  correctif. La 29-2 ne traite pas le départage. (#665, trop-perçu, devient la Story 29-6 : voisine, citée aux
  Exclusions.)
- **Écartées** : (a) traiter le départage dans la 29-2 — quatrième module (`kesh-reconciliation`), règle métier
  distincte ; (b) le laisser à la 29-3 — refusé par sa fiche ; (c) livrer la 29-2 seule en v0.13.4 — livrerait
  l'aggravation sans son correctif.
- **Réversible** : oui (ordre de merge et contenu de release).

### C-29-2-14 — La course « rappel annulé avant l'acceptation » est dite au manuel (`:2031`)

- **Contexte** : validation P2, R3 et F7. `:2031` dit « une facture proposée n'est plus refusée pour sa date » ; la
  course admise d'AC4 (un rappel annulé entre l'affichage et l'acceptation fait reculer l'ancre) rend de nouveau
  atteignable `payment_date_outside_window` sur une facture proposée et non redatée.
- **Retenu** : borner la phrase au refus « avant la facture » et dire, en une phrase, l'autre refus de date que la
  course rend possible (motif générique des *Échecs partiels*). Aucun test : la course reste écrite au doc-comment
  (AC4).
- **Écartées** : (a) taire la course (le manuel promettrait une absence de refus que le code ne tient pas) ; (b) un
  libellé d'écran propre à ce refus (texte d'écran, i18n, hors du périmètre minimal, C-29-2-6).
- **Réversible** : oui.

### C-29-2-15 — L'ancre est écrite par `macro_rules!`, en deux termes, pour que la 29-5 y insère l'exigibilité

- **Contexte** : consigne de l'orchestrateur au développement. La Story 29-5 (#668) réécrira l'ancre à partir d'une
  expression d'**exigibilité** `max(date, échéance)`. Deux `const &str` ne se composent pas (`concat!` n'accepte que
  des littéraux ; le dépôt n'a pas `const_format`).
- **Retenu** : l'ancre et sa jointure sont des `macro_rules!` (`invoice_payment_anchor_sql!`,
  `invoice_payment_anchor_join_sql!`), sur le patron de `accounts.rs` (`columns!`) et `invoices.rs`
  (`invoice_columns!`) ; les `const` `INVOICE_PAYMENT_ANCHOR_SQL` et `INVOICE_PAYMENT_ANCHOR_JOIN_SQL` en dérivent pour
  les requêtes `format!`, et la requête scalaire de l'acceptation est assemblée par `concat!` sur les mêmes macros.
  L'ancre s'écrit `CAST(GREATEST(GREATEST(i.date, COALESCE(i.due_date, i.date)), COALESCE(DATE(rmd.last_sent_at),
  i.date)) AS DATE)` — équivalente au `GREATEST` à trois termes de la fiche (`GREATEST` est associatif) —, dont le
  premier terme, sur sa propre ligne du `concat!`, est **l'exigibilité** : la 29-5 le remplace par l'appel de sa
  macro, sans toucher au terme du rappel, ni aux tests (qui n'assertent que des dates).
- **Écartées** : (a) un `const` à trois termes (non composable sans copie) ; (b) définir dès ici la macro
  d'exigibilité (préempterait le nom et la place que la 29-5 choisira, et ferait conflit au rebase) ; (c) ajouter
  `const_format` (dépendance neuve pour une seule composition).
- **Conséquence pour les mutations** : M1, M5 et M14 portent sur le premier terme (M1 le réduit à `i.date`, M14 à
  `COALESCE(i.due_date, i.date)`, M5 à `GREATEST(i.date, i.due_date)`), le `GREATEST` intérieur exigeant deux
  arguments.
- **Réversible** : oui.
- *(Annotation du 2026-10-14, revue de code P1, findings E1 = A2 (MEDIUM) et B2 (LOW) : M14 n'est **pas**
  équivalente — le repli `i.date` du terme du rappel ne vaut `F` que sans rappel ; une échéance et un rappel non
  annulé tous deux antérieurs à `F` font tomber l'ancre mutée sous `F`. Cas l ajouté à l'AC5 (fixture SQL : ce jeu
  est hors API), M14 rejouée et tuée ; doc-comment de `invoice_payment_anchor_sql!` corrigé.)*

### C-29-2-16 — Un `\label{sec:algorithme-matching}` neuf pour le renvoi du § des rappels

- **Contexte** : AC8 demande, à `:1305`, un renvoi « à `\S\ref{sec:facture-posterieure}` ou à l'algorithme » ; le
  § *Algorithme de matching* n'avait pas de label, et c'est lui qui dit la règle.
- **Retenu** : poser `\label{sec:algorithme-matching}` (libre au dépôt, `grep -rn algorithme-matching docs/manual`
  vide avant), et y renvoyer depuis `:1305` et la FAQ.
- **Écartée** : renvoyer à `sec:facture-posterieure` (il traite de la borne haute, pas de l'ancre).
- **Réversible** : oui.

### C-29-2-17 — Le coût de la dérivée des rappels (`rmd`, toutes sociétés) : angle mort écrit, sans filtre

- **Contexte** : revue de code P1, B3 = E5 (LOW). La dérivée `rmd` n'a pas de `company_id` : elle peut agréger les
  rappels de toutes les sociétés, une fois par transaction en attente (`get_proposals`). La justesse n'est pas en
  cause (jointure par `invoice_id`, `i.company_id = ?`, testé).
- **Retenu** : écrire l'angle mort au doc-comment de `invoice_payment_anchor_join_sql!`. MariaDB 10.11 a
  `split_materialized=on` (constaté sur `kesh-mariadb-dev`) et peut pousser la jointure dans la dérivée par l'index
  `idx_invoice_reminders_invoice` ; **non mesuré** (base de la story vide, plan sans valeur).
- **Écartée** : filtrer par société. Ce n'est pas mécanique : la dérivée ne peut pas référencer `i` (table dérivée,
  non latérale), le filtre exige un paramètre lié de plus, qui décale l'ordre des `bind` des deux lecteurs
  (proposition et scalaire) et quitte le patron sans paramètre des jointures du reste dû (C-29-2-12). Sans effet à
  l'échelle d'une installation à une société.
- **Réversible** : oui.

[#529]: https://github.com/guycorbaz/kesh/issues/529
[#666]: https://github.com/guycorbaz/kesh/issues/666
[#668]: https://github.com/guycorbaz/kesh/issues/668

---

## Story 29-3 — La proposition affichée suit une priorité écrite (#527)

*Registre propre à la story, à verser par union dans `epic-29-choix-autonomes.md`. Fiche :
`29-3-meilleure-proposition.md`. Choix faits le 2026-10-14 en autonomie : C-29-3-1 à 10 à la création de la fiche,
C-29-3-11 à 15 en validation P1 (C-29-3-11 tranché par l'orchestrateur), C-29-3-16 à 18 en validation P2, C-29-3-19 à 26 au développement.*

### C-29-3-1 — Une priorité écrite en trois rangs, et non le tri brut par score

- **Contexte** : #527 laisse le choix entre « candidates triées par score, toutes origines confondues » et « une règle
  de priorité écrite et dite à l'écran ». Le score de la règle (1,0) est une convention posée par `get_proposals`, non
  une mesure. Les factures « faibles » (< 0,5) sont toutes à 1–5 centimes du montant ; parmi elles, une facture dont le
  numéro figure dans la référence du paiement (0,4) est pratiquement certaine.
- **Retenu** : rang 0 = facture reconnue (montant au centime **ou** numéro dans la référence) ; rang 1 = la règle ;
  rang 2 = autres factures ; tri stable, l'ordre de `propose_matches` conservé à rang égal.
- **Écartées** : (a) tri brut par score — ferait passer la règle devant une facture référencée : produit compté deux
  fois, facture restée due, sans signal (le faux rattachement muet du `CLAUDE.md`), cas aujourd'hui bien servi ;
  (b) la règle toujours devant — idem ; (c) statu quo et une phrase à l'écran — la règle resterait cachée par une
  facture désignée par le seul nom du client, ce qui est le défaut de l'issue.
- **Réversible** : oui (une fonction pure, aucune donnée écrite).
- ⚠️ **Annotation du 2026-10-14 (validation P1, R1 HIGH)** : **rangs remplacés par C-29-3-11** (décision de
  l'orchestrateur). Le critère du rang 0 n'est plus « montant exact ou référence pleine » mais « tout signal
  d'identité » ; l'écartée (c) est en partie reprise : l'ordre atteignable reste celui d'aujourd'hui, la story l'écrit
  et le dit. Le texte ci-dessus est conservé comme relevé de la fiche initiale.

### C-29-3-2 — L'ordre se fixe au serveur, une seule fois ; le frontend lit `candidates[0]`

- **Contexte** : l'écran affiche et accepte `candidates[0]` du même tableau d'état.
- **Retenu** : `order_candidates` dans `get_proposals` ; le frontend inchangé sur ce point.
- **Écartée** : retrier au frontend (deux définitions de l'ordre, qui divergeraient ; un client d'API recevrait un
  autre ordre que l'écran).
- **Réversible** : oui.

### C-29-3-3 — Le rang 0 se lit sur les sous-scores, pas sur un seuil du total

- **Contexte** : les sous-scores valent des littéraux (`1.0`, `0.5`, `0.0`) ; le total est une somme de flottants.
- **Retenu** : `amount_score == 1.0 || reference_score == 1.0`.
- **Écartée** : `score.total >= 0.4` (équivalent aujourd'hui, mais illisible et fragile à tout changement de poids).
- **Réversible** : oui.
- ⚠️ **Annotation du 2026-10-14 (validation P1, C-29-3-11)** : le principe tient (sous-scores, jamais le total) ; la
  condition devient `amount_score == 1.0 || reference_score > 0.0 || contact_score > 0.0`.

### C-29-3-4 — La priorité est dite à l'écran par une phrase au-dessus de la liste

- **Contexte** : #527 demande qu'une priorité soit « dite à l'écran » ; le badge « 100 % » de la règle ou « 40 % »
  d'une facture affichée devant une règle cachée ne s'expliquent pas d'eux-mêmes.
- **Retenu** : clé `reconciliation-labels-priority-hint`, quatre locales, dans la branche « liste non vide », avant
  la table.
- **Écartées** : une infobulle par ligne (invisible au clavier et sur mobile, testée plus mal) ; une phrase hors de
  toute branche, comme l'indication des rejets (elle parle des propositions affichées : sans liste, elle ne dit
  rien de vrai).
- **Réversible** : oui.
- ⚠️ **Annotation du 2026-10-14 (validation P1)** : condition ajoutée — la phrase ne paraît que si une ligne a une
  candidate (C-29-3-14) ; formulation refaite (C-29-3-15).

### C-29-3-5 — Le seuil `INVOICE_OVERRIDE_THRESHOLD = 0.5` reste inchangé

- **Contexte** : AC #114 de la 8-5b ; une facture à 50 % ou plus supprime la règle de la liste.
- **Retenu** : inchangé. L'ordre suffit à corriger le défaut ; abaisser le seuil à 0,4 aurait le même effet à
  l'écran mais retirerait la règle de l'API, et change un critère d'une story livrée sans nécessité.
- **Réversible** : oui.

### C-29-3-6 — Le test muet `get_proposals_invoice_candidate_overrides_rule` est réparé dans la story

- **Contexte** : il ne crée aucune facture et n'éprouve donc pas l'effacement de la règle qu'annonce son nom
  (`reconciliation_rules_e2e.rs:961-1004`). La story repose sur ce comportement (C-29-3-5).
- **Retenu** : réparation **mécanique** (créer la facture, asserter l'absence de règle) — périmètre minimal respecté
  (B3 de la rétrospective 28).
- **Réversible** : oui.
- ⚠️ **Annotation du 2026-10-14 (validation P1, R2/F3)** : « mécanique » était faux — `reconciliation_rules_e2e.rs` n'a
  aucune aide de facture. Voie retenue : C-29-3-13.

### C-29-3-7 — Le trop-perçu de quelques centimes proposé puis refusé : issue, pas intégration

- **Contexte** : la tolérance de 5 centimes du dépôt charge une facture que la transaction **dépasse** de 1 à 5
  centimes ; l'acceptation la refuse en `RECONCILIATION_OVERPAYMENT` (`classify_payment`). Même famille que #548, sur
  le montant. Réparer touche le SQL de `kesh-db` et la tolérance, que la 29-2 retouche.
- **Retenu** : angle mort écrit à la fiche et au manuel ; **issue demandée** à l'orchestrateur (P2 proposée :
  proposition annoncée puis refusée, sans effet comptable). La story ne l'aggrave pas (une telle facture référencée
  passait déjà devant la règle).
- **Écartée** : intégrer (non mécanique ; conflit avec la 29-2 ; `kesh-db` touché imposerait le gate complet en
  boucle).
- **Réversible** : oui.
- ⚠️ **Annotation du 2026-10-14 (validation P1, R7/F6)** : l'issue existe, **#665** (ouverte, P2, bug). Sous C-29-3-11,
  une telle facture porte toujours un signal (sinon elle n'est pas candidate) : rang 0, toujours devant la règle. Les
  autres refus d'une facture proposée sont de configuration, déjà au manuel (`user-manual.tex:2056`).
- ⚠️ **Annotation du 2026-10-14 (validation P2, R2/F1, F9/R9)** : #665 est traitée par la **Story 29-6** (même
  release v0.13.4). Le contournement prescrit en P1 au manuel (« enregistrez alors le règlement depuis la fiche de la
  facture ») est **faux** : la fiche refuse aussi le trop-perçu (`SettleInvoiceDialog.svelte:179-184`,
  `user-manual.tex:1308`) — retiré, voir C-29-3-16. Et « tous de configuration » était trop fort : les autres refus
  sont aussi d'état comptable, de course ou d'intégrité (`:2060` du manuel).

- ⚠️ **Annotation du 2026-10-14 (passe ciblée P3, P3-3)** : « les autres refus … sont aussi d'état comptable, de course
  ou d'intégrité (`:2060` du manuel) » (annotation P2 ci-dessus) est faux pour le manuel : `user-manual.tex:2060` ne
  les énumère pas, et seuls les refus de **configuration** sont documentés (`:2056`). Les refus d'état, de course et
  d'intégrité ne sont **pas documentés** au manuel ; ni aggravés ni réparés par la 29-3, ils ne lui sont pas imputés.
### C-29-3-8 — `candidate_rank` est un `match` exhaustif, sans bras `_`

- **Contexte** : la 29-2 peut introduire un nouveau type de candidate.
- **Retenu** : un variant non classé ne compile pas — le classement devient une décision explicite au rebase.
- **Écartée** : un bras par défaut (un nouveau type prendrait un rang par accident, en silence).
- **Réversible** : oui.
- ⚠️ **Annotation du 2026-10-14 (validation P2, R6)** : la 29-2 **n'ajoute aucun** variant (sa fiche : ni source
  ni `CandidateType` neufs) ; c'est le choix manuel de #666 qui pourrait en ajouter un. Le choix tient pour lui.
- ⚠️ **Annotation du 2026-10-14 (revue de code P1, E1 LOW)** : l'exhaustivité du `match` garde un nouveau
  `CandidateType`, **non** une nouvelle valeur de sous-score. Le rang 2 reste inatteignable tant que
  `matching.rs:81` écarte les totaux nuls et que `amount_score` est binaire ; **si #666 ou L17 introduit un score de
  montant à gradient** (par exemple 0,9 pour un centime d'écart, total 0,45), une facture sans référence ni contact
  tomberait au rang 2, **derrière** la règle, sans que rien ne rougisse : aucun test ne parcourt les sorties réelles
  de `propose_matches` pour affirmer « jamais rang 2 ». Écrit **comme limite**, non corrigé (pas de code de production
  touché par la remédiation) ; à traiter avec la story qui introduit le gradient (condition de rang 0 à lire sur
  `amount_score > 0.0`, ou test de balayage).

### C-29-3-9 — L'écran reste à une candidate par ligne (KF-026, #76)

- **Contexte** : la vraie fin du masquage serait d'afficher toutes les candidates ; c'est #76, ouverte depuis la 8-4.
- **Retenu** : hors périmètre ; la story rend la candidate unique choisie selon une règle dite.
- **Réversible** : oui (#76 reste le lieu de cette évolution).

### C-29-3-10 — Les tests HTTP datent la facture du jour de la transaction

- **Contexte** : la 29-2 change la fenêtre de dates des propositions ; un test de la 29-3 qui en dépendrait casserait
  au rebase pour une raison étrangère.
- **Retenu** : facture et transaction du même jour, en plein exercice 2026 — dans toute fenêtre raisonnable.
- **Réversible** : oui.

### C-29-3-11 — Une facture qui porte un signal d'identité passe devant la règle (choix de l'orchestrateur)

- **Contexte** : validation P1, R1 HIGH — le rang 2 de C-29-3-1 plaçait la règle devant des factures que le nom du
  client (0,1) ou un préfixe de référence (0,2, 0,3) rapproche du paiement. Accepter la règle pour un paiement de
  client le porte en produit : produit compté deux fois, facture restée due, sans signal — le faux rattachement muet
  du `CLAUDE.md` (*Un appariement automatique propose, il ne crée jamais*), contre lequel le manuel met déjà en garde
  (`user-manual.tex:2911`). L'argument qui fondait le rang 0 vaut pour toute facture identifiée.
- **Retenu — choix de l'orchestrateur**, appliqué : rang 0 = facture avec `amount_score == 1.0 ||
  reference_score > 0.0 || contact_score > 0.0`, quel que soit le total ; rang 1 = la règle ; rang 2 = facture sans
  aucun signal (reconnue par le seul montant à quelques centimes).
- **Examen au code avant application** (consigne de l'orchestrateur) : **aucun défaut du produit démontré.** Une
  **conséquence démontrée** : une facture sans signal a un total nul et n'est pas candidate (`matching.rs:81`) ; quand
  la règle est calculée, aucune facture n'atteint 0,5, donc aucune n'a un montant exact, donc toutes portent un
  sous-score de référence ou de contact non nul. **Le rang 2 est vide sur toute entrée atteignable, et l'ordre affiché
  reste celui d'aujourd'hui.** La story n'ajoute donc aucun changement de comportement : elle écrit l'ordre (fonction
  pure, testée), le dit à l'écran, le documente pour l'API, et répare un test muet. L'appel `order_candidates` dans
  `get_proposals` est un mutant équivalent (M1), déclaré tel ; les mutations du **critère** (M4, M5, M7) restent tuées
  par les tests HTTP.
- **Risque résiduel nommé** : le faux rattachement symétrique (une contrepartie cliente et visée par une règle, dont le
  montant tombe à 5 centimes d'une facture ouverte) fait proposer la facture ; c'est l'ordre d'aujourd'hui, visible au
  lettrage.
- **Écartées** : (a) le rang 0 de C-29-3-1 (montant exact ou référence pleine) — écarté par l'orchestrateur ;
  (b) **ne pas écrire de fonction**, un commentaire juste à la place de celui de `:722-724` et les tests HTTP seuls —
  comportement identique et moins de code ; écartée parce que la priorité resterait un effet de la place du `push`
  (c'est ce que #527 relève), que le `match` exhaustif force le classement d'un variant que la 29-2 ajouterait
  (C-29-3-8), et que le rang 2 garde une source future de placer en silence une facture sans indice devant une
  règle. **Arbitrage réversible, signalé à l'orchestrateur** : si le surcoût (≈ 20 lignes, onze cas unitaires) paraît
  injustifié pour un ordre inchangé, (b) est l'alternative la plus proche.
- ⚠️ **Annotation du 2026-10-14 (validation P2) — arbitrage rendu : la fonction est GARDÉE**, décision de
  l'orchestrateur, les deux lentilles ayant confirmé l'argument principal (la priorité écrite, et non un effet de la
  place du `push`, est le mot de #527 ; `candidate_rank` serait le seul `match` sur `CandidateType`, donc le seul site
  qui fasse refuser la compilation d'un variant neuf). Deux raisons rectifiées (R6) : le variant neuf viendrait de
  #666, non de la 29-2 ; le rang 2 ne se peuplerait que par une source hors de `propose_matches` ou un sous-score non
  binaire, non par « un poids changé ». La mutation M11 éprouve désormais ce rang (F5). Le **risque résiduel** est
  élargi aux trois voies du code (contrepartie cliente visée par une règle ; référence courte ou préfixe commun ; nom
  contenu dans un autre), et « visible au lettrage » borné au cas où la facture choisie est celle du payeur réel (R7).
- **Réversible** : oui (une fonction pure, aucune donnée écrite).

### C-29-3-12 — Le départage à score égal par proximité de date : angle mort, traité par #668 (Story 29-5)

- **Contexte** : C-29-2-7 (fiche voisine non mergée) — la 29-2 fait entrer des factures échues et relancées ; à score
  égal, `propose_matches` place la plus petite `invoice_id` (la plus ancienne) en tête. Un départage par proximité de
  la date d'ancrage à la date du paiement est proposé.
- **Retenu** : non intégré ; angle mort écrit à la fiche et au doc-comment ; **issue demandée** à l'orchestrateur.
- **Écartée** : l'intégrer — non mécanique : la date d'ancrage est définie par la 29-2 (non mergée), le départage vit
  dans `kesh-reconciliation` (`matching.rs:97-105`), hors périmètre, et `order_candidates` ne réordonne pas à
  l'intérieur d'un rang (stabilité voulue).
- **Réversible** : oui.
- ⚠️ **Annotation du 2026-10-14 (validation P2, R3/F4)** : l'issue existe, **#668** (P2), traitée par la **Story
  29-5** dans la même release v0.13.4 ; attendu rectifié par l'orchestrateur selon l'art. 87 CO : à score égal, les
  factures **échues** d'abord, puis la plus ancienne. Le contexte ci-dessus recopiait une rédaction périmée de
  C-29-2-7 : la 29-2 fait entrer des factures échues, relancées **ou pas encore échues** — dont des factures anciennes
  à échéance lointaine, candidates sans limite de temps (C-29-2-11). La 29-3 ne traite pas ce départage et ne touche
  ni l'`ORDER BY` ni le `LIMIT 50` (C-29-3-17).

- ⚠️ **Annotation du 2026-10-14 (passe ciblée P3, P3-1)** : l'annotation P2 ci-dessus cite l'attendu de #668 d'après
  son premier commentaire (« échues d'abord, puis la plus ancienne »). La fiche **29-5**, qui le remplace, tranche : à
  score égal, **date d'exigibilité** `max(date de facture, échéance)` croissante, puis date de facture, puis
  identifiant (art. 87 CO). C'est cet attendu que la fiche 29-3 cite désormais ; le titre du choix, qui annonçait une
  « issue demandée », est corrigé (l'issue existe).
- ⚠️ **Annotation du 2026-10-14 (revue de code P1, B3 LOW)** : la doc de `order_candidates` et `docs/api-external.md`
  écrivent « puis par identifiant de facture croissant à score égal », vrai tant que `matching.rs:97-105` garde son
  `then_with(invoice_id)`. **#668 (Story 29-5) le remplace** : à son merge, relire ces deux phrases (déjà prescrit
  par C-29-3-26). Aucune action avant.
### C-29-3-13 — Le test muet est déplacé dans `reconciliation_e2e.rs`, non ses aides extraites

- **Contexte** : validation P1, R2/F3 — `get_proposals_invoice_candidate_overrides_rule` vit dans
  `reconciliation_rules_e2e.rs`, qui n'a aucune aide de facture ; celles-ci (`seed_validated_invoice` et ses
  dépendances, `:279-440`) sont privées à `reconciliation_e2e.rs`.
- **Retenu** : déplacer et réécrire le test dans `reconciliation_e2e.rs`, où vivent aussi les deux tests HTTP neufs de
  l'AC3 ; y ajouter `create_rule_ok` (une dizaine de lignes, patron `reconciliation_rules_e2e.rs:356-370`), dont les
  trois tests ont besoin ; un renvoi d'une ligne dans l'ancien fichier. Tâche T3, comptée comme test neuf.
- **Écartées** : (a) extraire les aides de facture vers `tests/common/` — ~150 lignes déplacées, tous les tests du
  fichier touchés, conflit assuré avec les tests de la 29-2 dans le même fichier ; (b) dupliquer les aides dans
  `reconciliation_rules_e2e.rs` — contraire au DRY du `CLAUDE.md`.
- **Réversible** : oui.

### C-29-3-14 — La phrase de l'écran n'apparaît que si une ligne a une candidate

- **Contexte** : validation P1, R10b/F8b — une liste dont toutes les lignes sont « Aucune correspondance » rendrait
  « une seule proposition est affichée par transaction » sans proposition.
- **Retenu** : condition `proposals.some((p) => p.candidates.length > 0)` dans la branche « liste non vide » ; un test
  Vitest et la mutation M9 la tiennent.
- **Écartée** : la phrase dès que la liste est non vide (C-29-3-4 initial) — texte creux dans ce cas.
- **Réversible** : oui.

### C-29-3-15 — La phrase de l'écran ne dit rien de plus fort que le code

- **Contexte** : validation P1, R5/F5 — `reference_score` vaut 1 quand la référence et le numéro se contiennent dans
  un sens **ou** dans l'autre, sans longueur minimale (`matching.rs:132-134`), et 0,5 pour un préfixe commun ; le
  contact est bidirectionnel aussi (`:148-161`). « Dont le numéro figure dans la référence » était plus étroit que le
  code.
- **Retenu** : « une facture passe devant une règle dès que sa référence, le nom de son client ou son montant exact la
  rapproche du paiement » (et l'équivalent dans les trois autres langues, termes de l'écran : *Zuordnungsregel*,
  *regola di assegnazione*, *reconciliation rule*).
- **Écartées** : « dont la référence et le numéro se contiennent l'un l'autre » — exact mais technique, et muet sur le
  préfixe et le contact ; ajouter une longueur minimale dans `matching.rs` — hors périmètre.
- **Réversible** : oui (une clé de catalogue).
- ⚠️ **Annotation du 2026-10-14 (validation P2, R1)** : la seconde phrase retenue ci-dessus affirmait une condition
  **suffisante** que le code ne donne pas — une facture n'est candidate que si son reste dû égale le montant à 5
  centimes près, dans la fenêtre de dates (`kesh-db` `reconciliation.rs:250-259`) ; un acompte dont la référence
  porte le numéro laisse la règle s'afficher. Remplacée, aux quatre langues, par une phrase **bornée à la facture
  proposée** : « Une facture proposée passe toujours devant une règle. » (de « Eine vorgeschlagene Rechnung geht
  einer Regel immer vor. », it « Una fattura proposta precede sempre una regola. », en « A proposed invoice always
  comes before a rule. »). La Story (« devant une facture proposée pour ce paiement ») suit.
- ⚠️ **Annotation du 2026-10-14 (revue de code P1, A1 et E2 LOW)** : la phrase d'écran, le CHANGELOG et le manuel
  disent « toujours » ; **le rang 2 est inatteignable** (aucune facture candidate n'a un total nul), donc la phrase est
  exacte sur tout ce que le code produit — **texte d'écran inchangé**. Deux limites écrites : (A1) le jour où un
  score à gradient rendrait le rang 2 atteignable (#666, L17), « toujours » deviendrait faux et la garde `i18n-keys`
  ne lit pas le sens — la phrase est à relire avec la condition de rang 0 (cf. annotation de C-29-3-8) ; (E2) le
  « signal d'identité » inclut des signaux faibles — sous-chaîne sans longueur minimale (`matching.rs:132-134`),
  préfixe commun de quatre caractères (0,5, `:139-143`), contact bidirectionnel sans minimum (L28) : l'écran et le
  manuel parlent de « rapprocher » sans promettre une reconnaissance solide, et l'ordre est celui d'avant la story
  (C-29-3-11). La longueur minimale n'est portée par aucune issue connue au sol.

### C-29-3-16 — Le manuel ne prescrit aucun contournement du trop-perçu de quelques centimes (#665)

- **Contexte** : validation P2, R2/F1. La remédiation P1 prescrivait au manuel « enregistrez alors le règlement
  depuis la fiche de la facture (issue \#665) » ; la fiche refuse le même trop-perçu (`SettleInvoiceDialog.svelte:
  179-184` ; `classify_payment` au serveur, `invoice_settlements_write.rs:217-228` ; manuel `:1308`). Seul le reste dû
  s'y enregistre, l'écart de quelques centimes n'est écrit nulle part, et la transaction reste à rejeter.
- **Retenu** : le manuel s'en tient au constat et au renvoi (« … son acceptation est refusée en trop-perçu (issue
  \#665) ») ; le contrôle du PDF vérifie l'absence du contournement. #665 est la Story 29-6, même release : au
  rebase, si elle est mergée, l'ajout s'adapte à ce qu'elle livre ou disparaît.
- **Écartée** : écrire le contournement complet (« saisir le reste dû depuis la fiche, puis rejeter la
  transaction ; l'écart n'est pas comptabilisé ») — exact au code, mais il enseignerait un geste qui laisse un écart
  de banque non comptabilisé, pour une durée de vie d'une release (la 29-6 le rend caduc).
- **Réversible** : oui (une phrase du manuel).

- ⚠️ **Annotation du 2026-10-14 (passe ciblée P3, P3-4)** : la raison de l'écartée, « la 29-6 le rend caduc », est
  inexacte. La fiche 29-6 garde ce geste (régler le reste dû depuis la fiche, puis **rejeter** la transaction, C-29-6-7)
  et l'écrit à son propre ajout du manuel (`:2049`) ; par décision de l'orchestrateur, elle rend possible le rejet d'une
  ligne **sans proposition**. La conclusion tient — la 29-3 n'écrit pas le contournement — mais parce que **la 29-6 le
  porte**, non parce qu'il serait caduc ; au rebase, la 29-3 s'efface devant ce que la 29-6 livre.
### C-29-3-17 — L'`ORDER BY` et le `LIMIT 50` du dépôt restent hors de la 29-3, la remarque va à la 29-5

- **Contexte** : validation P2, R3. La fiche 29-2 (version du 2026-10-14) annonce que la 29-3 « touchera probablement
  l'`ORDER BY` / `LIMIT 50` » ; au-delà de 50 candidates de même reste dû, `ORDER BY i.date ASC, i.id ASC` puis
  `LIMIT 50` (`kesh-db` `reconciliation.rs:258-259`) écartent les plus récentes, sans égard à l'exigibilité.
- **Retenu** : la 29-3 n'y touche pas (`kesh-db` hors périmètre, gate complet en boucle) ; la remarque est
  **transmise à la 29-5** (#668), qui ordonne les égalités et peut juger si la sélection avant `LIMIT 50` doit suivre
  la même règle — rapport à l'orchestrateur pour l'y noter.
- **Écartée** : l'intégrer ici — non mécanique, change un critère de la 28-7 (C-28-7-10), même famille que le
  départage.
- **Réversible** : oui.

### C-29-3-18 — L'E2E règle sa facture en `afterAll` si le parcours a échoué avant « Accepter »

- **Contexte** : validation P2, R8. Ce qui protège les autres specs de la base partagée, c'est le reste de 0,01
  laissé par l'acceptation, non le contact unique ; un échec avant « Accepter » laisse une facture due à 972,90,
  candidate à montant exact pour tout autre spec qui importe 972,90 dans sa fenêtre.
- **Retenu** : l'`afterAll` lit le reste dû et, s'il vaut 972,90, règle 972,89 par
  `POST /api/v1/invoices/{id}/settlements` ; le relevé du spec remplace toutes les références du gabarit par une
  valeur unique de préfixe `PRIO`, sans préfixe commun de quatre caractères avec un numéro de facture.
- **Écartées** : (a) écrire le risque sans parade — un échec isolé ferait rougir d'autres specs, d'un run à l'autre,
  loin de sa cause ; (b) une facture d'un autre montant — `createAndValidateInvoiceViaApi` a un TTC fixe, et une aide
  neuve élargirait le périmètre.
- **Réversible** : oui.

### C-29-3-19 — L'E2E tourne deux fois (clair, sombre), et nettoie en `finally`, pas en `afterAll`

- **Contexte** : développement, AC6. La recette visuelle de l'epic demande une capture claire et une sombre ; le
  patron du dépôt (`reconciliation-unreject.spec.ts`) fait tourner le scénario une fois par schéma, dans deux tests
  distincts.
- **Retenu** : deux tests, chacun avec sa facture, son relevé et sa règle ; le nettoyage prescrit par C-29-3-18
  (lire le reste dû ; s'il vaut 972,90, régler 972,89 ; supprimer la règle et l'import) est fait dans le `finally`
  de chaque test — il porte sur les identifiants de ce test, qu'un `afterAll` devrait collecter.
- **Écartée** : un `afterAll` commun — même effet, avec un état partagé entre les deux tests.
- **Réversible** : oui.
- ⚠️ **Annotation du 2026-10-14 (revue de code P1, B1 = E3 = A2 LOW)** : le `finally` d'origine appelait d'abord
  `amountDue` (qui jette) : s'il échouait, la règle, l'import et le lien du compte bancaire restaient, et l'erreur
  d'origine était masquée ; ni le règlement de secours ni la restauration du lien ne vérifiaient leur statut. Corrigé
  (E2E seul, aucune ligne de production) : **chaque nettoyage dans son propre `try`**, statut vérifié, échecs
  rapportés par `console.warn` et ne faisant échouer le test que si le parcours a réussi — l'erreur d'origine n'est
  jamais écrasée. ⚠️ **Le chemin de secours (règlement de 972,89 quand la facture reste due) n'a toujours pas été
  exercé** : les deux tests passent sans y entrer.
- ⚠️ **Annotation du 2026-10-14 (revue de code P1, E4 LOW)** : la ligne est cherchée dans la première page de
  `GET /proposals` (`DEFAULT_PROPOSALS_LIMIT = 100`, transactions lues par identifiant croissant) ; la limite est
  **écrite dans l'en-tête du spec**, non contournée (aucun cas constaté).

### C-29-3-20 — L'E2E lie le compte bancaire partagé au grand livre le temps du scénario, puis rend l'état d'origine

- **Contexte** : développement, premier run de l'AC6. Le compte de test `CH44…9012` de la base E2E n'a pas de
  `journal_account_id` : l'acceptation d'une facture y est refusée en `BANK_ACCOUNT_NOT_CONFIGURED` (constaté). Aucun
  spec existant n'accepte une facture sur ce compte, et aucun n'attend ce refus (`grep` des specs).
- **Retenu** : `linkToLedger` le lie à un compte de liquidités `10xx` s'il ne l'est pas, et `restoreJournalLink`
  rend la valeur d'origine dans le `finally` (vérifié en base après le run : `journal_account_id` revenu à `NULL`).
- **Écartées** : (a) un compte bancaire propre au spec — un IBAN valide de plus dans la base partagée, que d'autres
  specs (comptes bancaires) comptent ; (b) laisser le lien posé — un état changé en silence pour les autres specs.
- **Réversible** : oui.

### C-29-3-21 — L'E2E relit le nom enregistré du contact au lieu de le supposer

- **Contexte** : développement, premier run de l'AC6. `createContactWithAddressViaApi` crée une `Personne`, dont le
  nom enregistré est « Pia <nom> » (relevé en base) ; une contrepartie « <nom> Regle… » ne contenait donc pas ce nom,
  le score de contact valait 0, la facture n'était pas candidate et la ligne montrait la règle.
- **Retenu** : `GET /api/v1/contacts/{id}`, et la contrepartie du relevé porte le nom relu.
- **Écartée** : écrire « Pia » en dur — une hypothèse sur l'aide, qui se casserait sans le dire.
- **Réversible** : oui. *(À noter : `reconciliation-amount-due.spec.ts` écrit que sa contrepartie « porte le nom du
  contact » et compte sur ce score de contact pour départager ; il porte le même nom non composé — son départage par
  le contact n'a donc pas lieu. Ce spec ne s'appuie pas dessus pour passer ; remarque transmise à l'orchestrateur.)*

### C-29-3-22 — M9b n'est pas équivalente : un Vitest de plus, rechargement en erreur

- **Contexte** : campagne de mutations. M9b (phrase sortie de la branche « liste non vide », placée avant
  `{#if loading}`) **a survécu** aux tests de l'AC5 : au premier chargement, la condition « une ligne a une
  candidate » suffit. Mais `load()` ne vide pas `proposals` en cas d'erreur : un rechargement qui échoue après une
  liste à candidates afficherait la phrase au-dessus du message d'erreur.
- **Retenu** : test « rechargement en erreur après une liste à candidates ⇒ pas de phrase » (le rechargement qui
  suit « Accepter » échoue) ; M9b rejouée : tuée.
- **Écartée** : déclarer M9b équivalente — l'état observable existe.
- **Réversible** : oui.

### C-29-3-23 — Le renvoi du manuel vers la FAQ passe par un `\label` neuf

- **Contexte** : AC7, ajout de la raison à `:2049` avec renvoi à la FAQ (`:2911`). La sous-section de la FAQ n'avait
  pas d'étiquette.
- **Retenu** : `\label{sec:faq-aucun-match}` sur *Une transaction bancaire ne propose aucun match automatique, est-ce
  normal ?* ; le PDF rend « (§21.4) ».
- **Écartée** : citer la FAQ par son titre en toutes lettres — plus long, sans lien.
- **Réversible** : oui.

### C-29-3-24 — L'entrée du CHANGELOG va sous « Modifié », placé avant « Corrigé »

- **Contexte** : AC8. La section `[0.13.4] — Non publié` existait au rebase (créée par la 29-2), avec un seul
  sous-titre « Corrigé ». La fiche prescrit une entrée `Changed`.
- **Retenu** : sous-titre français « Modifié » (graphie des sections voisines), placé avant « Corrigé » comme dans la
  0.13.2 (« Ajouté », « Modifié », « Corrigé »).
- **Réversible** : oui.

### C-29-3-25 — Les captures sont versées dans `29-3-captures/` ; le sombre émulé rend comme le clair

- **Contexte** : AC6, recette visuelle. L'application n'a pas de thème sombre global (`grep` de
  `prefers-color-scheme` et `dark:` dans `frontend/src` : quelques classes `dark:` isolées, aucune sur la page de
  réconciliation) ; la capture en `colorScheme: 'dark'` est donc identique à la claire. C'est le constat, pas un
  défaut de la story : la phrase emploie le jeton `text-text-muted`, comme l'indication voisine des rejetées.
- **Retenu** : les deux PNG, `reconciliation-priorite-{clair,sombre}.png`, versés à côté de la fiche
  (`_bmad-output/implementation-artifacts/29-3-captures/`, 133 Ko au total), données synthétiques seulement.
- **Réversible** : oui (fichiers à retirer si l'orchestrateur préfère les garder hors dépôt).

### C-29-3-26 — `docs/api-external.md` écrit le départage d'aujourd'hui, et le `make fr` ne verse que le manuel utilisateur

- **Contexte** : AC8 et AC7. La 29-5 (#668) n'est pas mergée au rebase ; `make fr` régénère aussi le manuel
  d'administration et la brochure, dont les sources n'ont pas bougé (seuls leurs octets de génération diffèrent).
- **Retenu** : « par identifiant de facture croissant à score égal », à relire au merge de la 29-5 ; les PDF
  d'administration et de brochure régénérés sont rendus à leur version commitée (`git checkout --`, dans le
  worktree), seul `user-manual.pdf` est versé — comme la 29-2.
- **Réversible** : oui.

### C-29-3-27 — `reconciliation-amount-due.spec.ts` : le commentaire dit ce que le spec prouve, le montage ne change pas

- **Contexte** : revue de code P1, défaut voisin relevé par le développement (C-29-3-21) et tranché par
  l'orchestrateur. Ce spec écrit que la contrepartie de son relevé « porte le nom du contact » et que le score de
  contact départagerait une autre facture de même reste ; or la contrepartie reprend le nom **saisi** (`Solde <n>`),
  non le nom **enregistré** d'une `Personne` (« Pia Solde <n> »), que ce spec ne relit pas.
- **Retenu** : réécrire le commentaire (en-tête du spec, E2E seul) pour dire ce qu'il prouve — la facture est
  candidate sur son reste dû, tenue seule par l'unicité du reste — et qu'il **ne prouve pas** le départage par le
  contact. Montage inchangé.
- **Écartée** : relire le nom par l'API comme C-29-3-21 — plus de code dans un spec d'une autre story pour un
  départage que l'objet du spec (le reste dû) n'exige pas, et que les e2e API couvrent.
## Story 29-1 — Préparer la clôture d'un exercice (#232)

### C-29-1-1 — Une fiche, deux stories (29-1a contrôles préalables, 29-1b transitoires) ; pas de 29-1c

- **Contexte** : l'ensemble touche plus de cinq modules (§ *Règle de splitting préventif*) ; le fichier d'epic
  prévoyait 29-1a / 29-1b / « éventuellement 29-1c affectation du résultat ».
- **Retenu** : précédents C-26-2-15 et C-28-4-1 — une fiche, deux clés, deux PR successives, clé mère `split`.
  29-1a (lecture seule, `refs #232`) puis 29-1b (contre-passation au début de l'exercice suivant, E2E de parcours,
  répétition à blanc au manuel d'administration, `closes #232`). Chacune compte cinq modules, la borne de la règle.
  Pas de 29-1c : l'affectation du résultat est écartée (C-29-1-2) ; la répétition à blanc va à la dernière
  sous-story, la 29-1b, comme le demande l'orchestrateur.
- **Écartées** : une story unique (trop large) ; trois stories avec une 29-1c « répétition à blanc » seule
  (documentation sans code, qui se relit avec la 29-1b dont elle décrit les gestes).
  *(Rectifié en validation P2, 2026-10-14, décision de l'orchestrateur : la répétition à blanc **quitte** la 29-1b et
  devient la Story 29-7 — C-29-1-23. Le découpage 29-1a / 29-1b tient.)*
- **Réversible** : oui.

### C-29-1-2 — Ni écritures de clôture, ni écriture d'affectation du résultat

- **Contexte** : #232 demande la « génération des écritures de clôture (virement du résultat au bilan) » ;
  l'orchestrateur propose d'examiner une écriture d'affectation du résultat. Le modèle « temps réel virtuel »
  (Story 14-1, décision de Guy du 2026-07-21) calcule résultat de l'exercice et résultat reporté ; le compte 2979
  (`CurrentYearResult`) est non imputable.
- **Retenu** : rien n'est généré. Une écriture de virement doublerait le calcul et fausserait le bilan. Pour une
  personne physique ou un indépendant, l'affectation est sans objet (le résultat s'accumule aux fonds propres, ce que
  la ligne « Résultat reporté (calculé) » montre). Pour une société, l'affectation votée par l'assemblée reste une
  écriture **manuelle**, déjà possible (débit du compte physique de rôle `RetainedEarnings`, crédit dividendes ou
  réserve) ; sa **présentation** au bilan (compte physique négatif à côté de la ligne calculée positive) est
  perfectible — **demande d'issue** à l'orchestrateur, pour que la fermeture de #232 ne la fasse pas disparaître.
  *(Validation P1, 2026-10-14 : l'issue existe — **#667**, « Affectation du résultat d'une société — écriture de
  décision d'assemblée et présentation au bilan », `enhancement`, sans label de priorité à cette date : à poser.)*
- **Écartées** : écriture d'affectation assistée (double un calcul, ou exige de revoir le modèle 14-1) ; retour à des
  écritures de clôture physiques (décision d'architecture contraire).
- **Réversible** : oui (une story future peut ajouter l'affectation assistée pour les sociétés).

### C-29-1-3 — Les contrôles signalent, ils ne bloquent pas ; `fiscal_years::close` n'est pas touchée

- **Contexte** : candidats de l'orchestrateur (contrôles en lecture seule, utilisables en cours d'année) ; leçon B5
  de la rétrospective 28 (toute simplification qui touche une transaction se relit sur les verrous et la naissance de
  la vue).
- **Retenu** : une lecture séparée, rappelée dans le dialogue de clôture, sans effet sur le `POST /close`. Le bouton
  *Clôturer* reste actif ; seul K1 (antérieur ouvert) bloque, et il bloquait déjà. Aucun verrou, aucune lecture ne
  s'ajoute à la clôture.
- **Écartées** : refuser la clôture tant qu'un contrôle est en `attention` (la clôture se défait par une réouverture
  motivée ; un refus côté serveur obligerait à lire les contrôles **sous** les verrous de la clôture — exactement le
  terrain de la 28-4) ; une case « j'ai vu les avertissements » envoyée au serveur (cérémonie sans garantie).
- **Réversible** : oui.

### C-29-1-4 — Sept contrôles, et ceux qui sont écartés

- **Retenu** : K1 antérieur ouvert, K2 transactions bancaires non rapprochées, K3 relevé de fin d'année (manquant,
  arrêté trop tôt, en écart), K4 factures en brouillon, K5 factures fournisseur importées à compléter, K6 écritures
  incohérentes héritées (déséquilibrées, moins de deux lignes, date hors de leur exercice, ou d'un autre exercice
  datées dans celui-ci), K7 exercice suivant absent.
- **Écartés**, un par un :
  - *comptes d'attente soldés* : Kesh n'en a pas (aucun rôle, drapeau ni compte ; inventaire au code) ; un rôle neuf
    serait une migration breaking (un binaire antérieur ne décode pas un rôle inconnu) — prémisse de #232 corrigée ;
  - *TVA réconciliée* : report assumé des P1 de la TVA (utilisateur non assujetti) ; demande d'issue ou de
    rattachement au jalon « Vague 2 » *(validation P1, 2026-10-14 : noté en commentaire sur **#398**, jalon « Vague
    2 », sans issue neuve)* ;
  - *équation du bilan* : K6 en couvre les deux causes (écriture déséquilibrée ; ligne datée dans un exercice et
    rattachée à un autre — le compte de résultat filtre `fiscal_year_id`, le bilan non) ; un contrôle de l'équation
    répéterait K6 sans dire quelle écriture corriger ;
  - *verrou de période contre fin d'exercice* : la clôture fige tout l'exercice, le verrou n'ajoute rien à signaler ;
  - *postes ouverts débiteurs / créanciers* : une créance ouverte au 31 décembre est normale, l'écran *Postes
    ouverts* les liste déjà à une date ;
  - *factures fournisseur saisies ouvertes* : déjà comptabilisées (dette), rien d'anormal.
- **Réversible** : oui (un contrôle s'ajoute par un changement de la fiche et du test d'ensemble clos T-a1).

### C-29-1-5 — Lecture sous `ReadSnapshot`, tout rôle, sans audit

- **Contexte** : sept requêtes qui doivent voir la base au même instant ; les rapports lisent sur le pool sans
  instantané ; `ReadSnapshot` (Story 28-3) est conçu pour être réutilisé.
- **Retenu** : une connexion, une vue, `READ ONLY` ; route dans `authenticated_routes` (Consultation lit, comme les
  rapports) ; pas d'audit (lecture, pas un export).
- **Écartées** : lectures sur le pool (incohérence possible entre contrôles) ; audit `fiscal_year.checks_viewed`
  (bruit dans une piste de contrôle).
- **Réversible** : oui.

### C-29-1-6 — Le module `kesh-db/src/repositories/closing_checks.rs` et la route sous `/fiscal-years/{id}`

- **Retenu** : la lecture vit à côté des dépôts qu'elle interroge ; la route est une ressource de l'exercice.
- **Écartées** : `kesh-report` (états comptables, pas contrôles de données) ; `/reports/closing-checks?fiscalYearId=`
  (ce n'est pas un rapport exportable).
- **Réversible** : oui.

### C-29-1-7 — K3 réutilise la requête du tableau de bord, bornée, sans copie

- **Contexte** : seule `list_by_company_with_balances` sait corriger l'écart des décalages date de valeur /
  comptabilisation ; elle ne prend pas de date.
- **Retenu** : extraction d'une fonction privée au SQL **unique**, avec une borne facultative sur `period_to` ; le
  tableau de bord passe `None`, résultat inchangé (tests existants verts sans modification).
- **Écartées** : copier la requête (règle DRY ; deux copies divergeraient au prochain correctif) ; comparer seulement
  le dernier relevé sans borne (un relevé de janvier N+1 masquerait l'absence de celui de décembre).
- **Réversible** : oui.

### C-29-1-8 — Les transitoires : un geste « contre-passer au début de l'exercice suivant », sans marque ni migration

- **Contexte** : #232 demande « saisie des actifs/passifs de régularisation avec contre-passation automatique à
  l'ouverture de N+1 ». Kesh n'a pas d'événement « ouverture » (l'exercice suivant est créé par l'utilisateur, souvent
  avant la clôture de N) ; la contre-passation existe, datée du jour.
- **Retenu** : l'écriture de régularisation se saisit comme toute écriture ; un bouton de sa fiche la contre-passe au
  **premier jour de l'exercice qui suit le sien**, par la contre-passation unique (liaison `reverses_entry_id`,
  lettrage, refus, rejeu). « Automatique » s'entend : Kesh construit, date et lie l'inverse ; l'utilisateur déclenche.
- **Écartées** : (a) marque « transitoire » à la saisie + génération à la clôture de N — écrit sous les verrous de la
  clôture (cycle neuf avec les écrivains de N+1, terrain de la 28-4 et de #651), migration, et génération en masse qui
  décide seule ; (b) marque à la saisie + création de l'inverse dans la même transaction — l'inverse naîtrait avant
  que l'origine soit relue, et l'exercice suivant doit exister au moment de la saisie ; (c) contre-passation à date
  libre (C-29-1-9).
- **Réversible** : oui (une marque pourra s'ajouter plus tard, la machinerie restera).

### C-29-1-9 — Une seule date neuve : le premier jour de l'exercice suivant

- **Contexte** : le manuel fonde le verrou de période sur « la contre-passation est datée du jour : elle corrige
  depuis le présent ».
- **Retenu** : `ReversalDating { Today, NextFiscalYearStart }`. La date neuve tombe dans un exercice **postérieur** à
  celui de l'origine et passe par **tous** les refus de `create_in_tx_inner` (période verrouillée comprise).
- **Écartées** : une date choisie par l'utilisateur (rouvrirait la question du verrou de période, contre-passation
  datée dans le passé de l'origine).
- **Réversible** : oui.

### C-29-1-10 — Une route neuve, pas un corps sur `/reverse`

- **Retenu** : `POST /api/v1/journal-entries/{id}/reverse-at-next-fiscal-year-start`, sans corps.
- **Écartées** : un corps facultatif `{ "dating": … }` sur `/reverse` — les clients actuels envoient `{}` ou rien,
  et un extracteur JSON rendu obligatoire les casserait ; un paramètre de requête (un geste d'écriture distinct mérite
  une route nommée, inscrite au registre et rejouée).
- **Réversible** : oui.

### C-29-1-11 — L'exercice suivant : lecture ordinaire après `reversal_blockers`, confirmée par lecture verrouillante

- **Contexte** : B5 et § 4.2 de la rétrospective 28 — une lecture ordinaire avancée fait naître la vue plus tôt.
- **Retenu** : la vue naît déjà à `reversal_blockers` (première lecture ordinaire de la contre-passation) ; la lecture
  de l'exercice suivant se place **après**, au point où `today` est calculé ; `find_open_covering_date` (verrouillante,
  état validé le plus récent) confirme. Angle mort écrit au doc-comment : un exercice **intercalé** créé après la vue
  n'est pas vu.
  *(Rectifié en validation P1, 2026-10-14, F4 / R14 : « première lecture ordinaire » n'est vrai que **sur ce chemin**,
  appelé avec `authority = None` ; avec une autorité fournisseur, l'étape (1-bis) est une lecture ordinaire
  antérieure. La phrase ne s'énonce plus en général — AC11 de la fiche. Et la **sûreté** repose sur la confirmation
  verrouillante de N+1, non sur le verrou de N — AC12, C-29-1-20.)*
- **Écartées** : une lecture verrouillante par intervalle pour trouver le suivant (ordre du parcours fixé par
  l'optimiseur, verrous de clé suivante — la doc de `fiscal_years::close` explique pourquoi la clôture y a renoncé).
- **Réversible** : oui.

### C-29-1-12 — Refus `NEXT_FISCAL_YEAR_MISSING` et `NEXT_FISCAL_YEAR_CLOSED`, en 409

- **Retenu** : deux codes distincts — l'un appelle à créer l'exercice, l'autre à le rouvrir ou à renoncer —, 409
  comme les autres refus d'état d'exercice (`EARLIER_FISCAL_YEAR_OPEN`).
- **Écartée** : réutiliser `FISCAL_YEAR_INVALID` (400, « aucun exercice ouvert ne couvre cette date ») — ne dit ni
  quelle date ni quoi faire.
- **Réversible** : oui (codes neufs, aucun client ne les attend encore).

### C-29-1-13 — E2E de parcours et répétition à blanc dans la 29-1b

- **Contexte** : Guy ne testera le bouclement qu'en début 2027 ; aucune recette en usage ne rattrapera ce que les
  revues manquent.
- **Retenu** : un E2E de parcours complet (contrôles, rapprochement, transitoire, contre-passation, bilans N et N+1,
  clôture, réouverture) ; au manuel d'administration, une marche de répétition sur une **instance séparée** à partir
  d'un `.keshbackup`, sans rien rapporter vers la production.
  *(Rectifié en validation P2, 2026-10-14 : l'E2E de parcours reste dans la 29-1b ; la répétition à blanc en est
  retirée — Story 29-7, C-29-1-23.)*
- **Réversible** : oui.

### C-29-1-14 — Angle mort assumé : une transitoire oubliée n'est pas repérée

- **Contexte** : sans marque (C-29-1-8), rien ne distingue une écriture de régularisation d'une autre.
- **Retenu** : écrit au manuel (AC17). Pas une dette au sens du `CLAUDE.md` (aucun défaut : une fonctionnalité non
  promise) ; si Guy la veut, CR à ouvrir.
- **Réversible** : oui.

### C-29-1-15 — Le défaut voisin « (si clôturé) » du manuel est corrigé dans la 29-1a

- **Contexte** : `user-manual.tex:728` affirme que les comptes du bilan sont reportés « depuis le bilan de clôture de
  l'exercice précédent (si clôturé) » ; au code, le report est calculé que l'exercice précédent soit clos ou non.
- **Retenu** : correction **mécanique** (une parenthèse retirée, une phrase juste), dans le commit du manuel de la
  29-1a (B3 : un défaut voisin mécanique s'intègre).
- **Réversible** : oui.

---

## Choix ajoutés à la remédiation de la validation P1 (2026-10-14, Opus 5.5)

### C-29-1-16 — K3 : population par l'activité, précédence écrite, comptes non vérifiables sans effet sur le statut

- **Contexte** : P1 R5 — la population d'origine (tout compte non archivé ni clos avant Y) faisait signaler chaque
  année un compte alimenté par CSV seulement, un compte ouvert après Y et un compte sans mouvement ; la précédence
  entre « sans relevé » et « non comparable » n'était pas écrite.
- **Retenu** : compte examiné = non archivé, non clos avant Y, **avec une activité dans Y** (une transaction bancaire
  ou un relevé à solde daté dans Y). Motif = le premier de `NOT_COMPARABLE` > `NO_BALANCE_STATEMENT` > `MISSING` >
  `ENDS_BEFORE_YEAR_END` > `GAP`. Les deux premiers vont dans une liste `notCheckable`, affichée, **sans effet sur le
  statut** ; les trois suivants font l'`attention`.
  *(Rectifié en validation P3, 2026-10-14, P3 R5 : la définition de l'activité ci-dessus est **supplantée** par
  C-29-1-24 — une ligne d'écriture sur le compte de grand livre lié compte aussi ; et la source du motif
  `NO_BALANCE_STATEMENT` est précisée à la fiche (quatrième `EXISTS`, sans borne de date, P3 R8).)*
- **Écartées** : un critère `created_at <= Y.end_date` (faux pour une reprise de comptabilité : le compte se crée en
  N+1 et reçoit les relevés de N) ; exclure purement les comptes non vérifiables (« signaler, jamais taire » : on
  dit pourquoi un compte n'est pas contrôlé) ; `NOT_COMPARABLE` en `attention` (alerte permanente pour un partage
  légitime d'un compte de grand livre).
- **Réversible** : oui.

### C-29-1-17 — L'écart du relevé : une seule définition côté serveur, arrondie au centime comme le tableau de bord

- **Contexte** : P1 R6 — les lignes sont en `DECIMAL(19,4)`, le relevé en `DECIMAL(18,2)` ; le tableau de bord arrondit
  les deux termes au centime (`statementGap`, TypeScript), K3 comparait sans arrondi.
- **Retenu** : `bank_accounts::statement_gap` (Rust), arrondi par `Money::round_to_centimes` ; le panneau affiche la
  valeur du serveur sans recalcul ; le test Rust reprend les cas de `homepage.test.ts`. Le calcul TypeScript du
  tableau de bord reste (servir son écart depuis le serveur toucherait un sixième module) : **issue demandée**.
  *(Rectifié en validation P2, 2026-10-14, P2 R8 : l'argument « sixième module » était faux au grain où la fiche
  compte, et l'issue n'existait pas. Le tableau de bord lit désormais l'écart du serveur — C-29-1-25 ; la seconde
  définition disparaît, la demande d'issue tombe.)*
- **Écartées** : recopier `statementGap` dans le panneau (troisième définition) ; comparer sans arrondi.
- **Réversible** : oui.

### C-29-1-18 — Lectures d'exercice partagées : `EARLIER_OPEN_SQL` et `find_next_after`

- **Contexte** : P1 R13 — K1 recopiait le prédicat de `FIND_EARLIER_OPEN_SQL` ; K7 et l'étape (4) de la 29-1b
  cherchent le même « exercice suivant ».
- **Retenu** : `fiscal_years.rs` porte le texte sans verrou (`EARLIER_OPEN_SQL`, `NEXT_AFTER_SQL`) et ses lectures
  (`find_earliest_earlier_open`, `find_next_after`), sur le modèle `OPEN_COVERING_DATE_SQL` ; `close` exécute le
  même texte suffixé de `FOR UPDATE`, identique octet pour octet (test T-a15). K7 et la contre-passation appellent
  la même fonction.
- **Écartées** : deux requêtes voisines dans `closing_checks.rs` et `journal_entries.rs`.
- **Réversible** : oui.

### C-29-1-19 — Le rappel du dialogue de clôture compte les contrôles `info`

- **Contexte** : P1 R16 — K7 en `info` ferait dire « 1 point à vérifier » tant que N+1 n'existe pas.
- **Retenu** : tous les `attention` comptent : un exercice suivant absent empêche de contre-passer les transitoires,
  des brouillons et des pièces à compléter sont des points réels avant une clôture. `notCheckable` ne compte pas.
- **Écartée** : ne compter que `blocking` / `warning` / `anomaly` (masquerait K4, K5, K7 au moment même où ils
  servent).
- **Réversible** : oui.

### C-29-1-20 — T-b5 : la vraie fonction en transaction, deux ordres, N premier exercice

- **Contexte** : P1 F1 / F2 / R14 — « A s'arrête après l'étape (1) » n'était pas montable ; avec un exercice
  antérieur à N, la clôture de N+1 forme le cycle connu au lieu d'attendre ; aucune mutation n'était tuée par T-b5.
- **Retenu** : `reverse_at_next_fiscal_year_start_in_tx` public ; ordre (i) A va au bout sans valider, B (`close`)
  attend en (b') — test de comportement, ne tue aucune mutation (N tenu deux fois), déclaré ; ordre (ii) B monté à la
  main (effets de `close`), A (vraie fonction) attend à l'étape (1) puis rend `NEXT_FISCAL_YEAR_CLOSED` — tue M-b3 et
  M-b11. N est le premier exercice. L'AC12 et le doc-comment disent que la **sûreté** repose sur la confirmation
  verrouillante de N+1 ; le verrou de N n'en est qu'une conséquence.
  *(Rectifié en validation P3, 2026-10-14, P3 R5 : l'ordre (ii) **ne tue plus M-b3** — voir C-29-1-27 ; M-b3 est tuée
  par T-b3 seul. La mutation M-b11 est réécrite à la fiche : c'est le `JOIN fiscal_years fy` qui sort de la requête
  verrouillante de l'étape (1), P3 F2.)*
- **Écartées** : reconstituer à la main les étapes de la contre-passation (précédent du test 13 d — ne tue aucune
  mutation du code) ; ajouter une variante en transaction de `close` (hors périmètre, touche la clôture).
- **Réversible** : oui.

### C-29-1-21 — Répétition à blanc : marche exécutable, et le `.keshbackup` traité comme un secret

- **Contexte** : P1 R8 / F7 — `container_name` et ports littéraux, image `:latest`, compose de production à base
  externe, SMTP de production qui enverrait de vrais courriels, fichier d'export qui porte condensés de mots de passe,
  clés API et jetons.
- **Retenu** : autre machine de préférence ; sinon dossier distinct, copie de `docker-compose.yml` (jamais
  `docker-compose.prod.yml` avec le `DATABASE_URL` de production), `container_name` et port édités, image épinglée à
  la version de `/health`, `-p kesh-repetition`, `.env` propre sans `KESH_SMTP_*` ; à la fin, `down -v`, suppression
  du dossier (avec sa sauvegarde pré-import) **et** du `.keshbackup` exporté ; jamais de dépôt dans un dossier
  synchronisé ni un dépôt de sources ; chiffrement s'il quitte la machine.
- **Écartées** : « un autre projet docker-compose » sans plus (non exécutable) ; garder l'export « pour la prochaine
  fois » (un secret qui traîne).
  *(Caduc en validation P2, 2026-10-14 : l'AC19 est retirée de la fiche — C-29-1-23. Ce choix reste comme trace et
  comme point de départ de la Story 29-7, qui devra l'éprouver en l'exécutant ; P2 R1 et R2 montrent qu'il ne suffit
  pas en l'état.)*
- **Réversible** : oui.

### C-29-1-22 — Fiche d'écriture : un motif à la place du bouton absent

- **Contexte** : P1 R9 — la règle de la page (« les boutons sont absents… le motif est affiché à leur place ») ;
  l'oubli de créer N+1 est le cas d'usage central.
- **Retenu** : motif traduit selon l'état de la liste des exercices (aucun pendant le chargement ; indisponibilité sur
  échec ; « créez d'abord l'exercice suivant » ; « l'exercice suivant est clôturé »), Admin/Comptable seulement.
  L'échec de la liste ne retire pas *Contre-passer* (du jour).
- **Écartées** : bouton désactivé (la page refuse les boutons grisés) ; bouton affiché sans liste (la confirmation
  doit nommer la date et l'exercice).
- **Réversible** : oui.

---

## Choix ajoutés à la remédiation de la validation P2 (2026-10-14, Opus 5.5)

*C-29-1-23 à 26 consignent des **décisions de l'orchestrateur**, transmises dans le prompt de remédiation ; C-29-1-27
à 29 sont des choix de la remédiation.*

### C-29-1-23 — La répétition à blanc quitte la fiche : Story 29-7 (décision de l'orchestrateur)

- **Contexte** : signal D5 levé en P2 (P1 0 HIGH / 12 MEDIUM → P2 1 HIGH / 12 MEDIUM, 13 des 16 findings R nés de la
  remédiation P1). L'AC19 a produit un défaut à chaque correction (P1 R8, F5, F7 ; P2 R1 HIGH, R2, F5), dont un qui
  pouvait faire remplacer la base de production par l'import (sur Synology, le compose de production s'enregistre
  sous le nom `docker-compose.yml`).
- **Retenu** : l'AC19, ses tâches et ses textes prescrits du manuel d'administration sont **retirés** — non réécrits ;
  l'AC19 garde son titre en renvoi. Story **29-7** (doc), clé `29-7-repetition-a-blanc: backlog`, à écrire **en
  l'exécutant réellement** sur la station avec une sauvegarde synthétique ; cahier des charges : P2 R1, R2 et F5.
- **Écartées** : une troisième réécriture de la marche dans la fiche (le motif du dépôt : chaque remédiation y
  produisait le défaut suivant, et seule l'exécution le révèle) ; une 29-1c doc (la marche se relit mieux exécutée
  qu'adossée à une fiche de code).
- **Réversible** : oui.

### C-29-1-24 — K3 lit aussi le grand livre (décision de l'orchestrateur)

- **Contexte** : P2 R3 = F2, convergents — un compte mouvementé au grand livre sans import bancaire dans Y échappait
  à K3, alors que son solde de fin d'année n'est confronté à aucun relevé.
- **Retenu** : compte examiné s'il a, dans Y, une transaction bancaire, un relevé à solde **ou** une ligne d'écriture
  sur son compte de grand livre lié ; trois `EXISTS` séparés (jamais `bank_transactions` dans le même `FROM` que les
  lignes) ; cas de T-a5, mutation M-a20, phrase du manuel écrite telle quelle.
- **Écartée** : écrire l'angle mort au manuel (« activité bancaire seulement ») — il tait le cas même que K3 doit
  signaler.
- **Réversible** : oui.

### C-29-1-25 — Le tableau de bord lit l'écart servi par le serveur (décision de l'orchestrateur)

- **Contexte** : P2 R8 — l'argument « sixième module » de C-29-1-17 était faux au grain de la crate, et l'issue
  demandée n'existait pas ; deux définitions de l'écart (Rust, TypeScript) restaient.
- **Retenu** : champ `statement_gap` dans `BankAccountWithBalance`, lu par `bank-accounts.api.ts` ; `statementGap` de
  `homepage.ts` lit ce champ, `toCentime` disparaît ; les cas d'arrondi passent en Rust (T-a14), test de route et de
  lecture neufs (T-a16), mutation M-a21. Une seule définition. Aucune crate neuve ; dispersion au grain fin déclarée.
- **Écartée** : garder le calcul TypeScript et ouvrir une issue (deux définitions qui divergeraient au prochain
  correctif).
- **Réversible** : oui.

### C-29-1-26 — T-a2 inversée : inventaire des sites non résolus (décision de l'orchestrateur)

- **Contexte** : P2 R4 = F1 — la garde « aucun `query*("` littéral » manquait le turbofish, l'appel coupé par
  rustfmt, `r#"…"#`, `concat!` et `format!` : verte sur la forme exacte qu'elle devait interdire.
- **Retenu** : tout site d'appel de requête du module a pour premier argument une constante de `ALL_SQL` (turbofish
  sauté en équilibrant les chevrons, source mis sur une ligne) — tout autre site est nommé comme non résolu ; recherche
  de `FOR UPDATE` / `LOCK IN SHARE MODE` / `FOR SHARE` dans le source hors tests **décommenté** ; `format!` et
  `concat!` en argument de requête sont des non résolus (non interdits ailleurs : un libellé peut s'y construire) ;
  liste close des fonctions externes ; cas de contrôle du détecteur ; mutation M-a19.
  *(Rectifié en validation P3, 2026-10-14, **décision de l'orchestrateur** — P3 R2, signal D5 au troisième tour sur
  cette garde : « une forme qui produit un défaut à chaque correction se retire ». T-a2 est **réduite à ses deux
  volets solides** : (a) tout appel de requête a pour premier argument une constante de `ALL_SQL` ; (b) aucun mot de
  verrou dans le source décommenté. Les volets (c) — liste close des fonctions externes, que les noms importés par
  `use` contournaient et qu'aucun cas de contrôle n'exerçait — et (d) — constantes externes sans verrou — **sortent
  du test**, sans rien qui les remplace : les fonctions de dépôt externes appelées par `closing_checks` sont nommées
  dans son doc-comment et vérifiées en revue de code, **angle mort assumé** écrit à l'AC2. Conséquence mécanique :
  `BALANCES_SQL`, `EARLIER_OPEN_SQL` et `NEXT_AFTER_SQL`, rendues `pub(crate)` pour le seul volet (d), redeviennent
  privées.)*
- **Écartée** : ajouter des formes à l'énumération (la classe resterait ouverte).
- **Réversible** : oui.

### C-29-1-27 — M-b3 : le statut de N+1 n'est jugé que par la confirmation ; T-b5 (ii) ne la revendique plus

- **Contexte** : P2 R5 = F3 — une implémentation qui juge `next.status` sur la lecture ordinaire rend
  `NEXT_FISCAL_YEAR_CLOSED` avec ou sans confirmation, dans T-b3 comme dans T-b5 (ii) ; l'état qui distinguerait la
  confirmation (« ordinaire : ouvert ; verrouillante : clos ») est inatteignable sous `close` actuel. L'orchestrateur
  laissait le choix entre un ordre (iii) où B ne tient que N+1 et le retrait de T-b5 (ii) des tueurs de M-b3.
- **Retenu** : l'AC11 **prescrit** que le statut n'est jugé que par la confirmation verrouillante ; M-b3 (confirmation
  retirée, aucun contrôle) est tuée par T-b3 seul ; T-b5 (ii) ne tue que M-b11 ; la variante « statut lu sur la
  lecture ordinaire » est M-b13, **déclarée équivalente**, exclue par la prescription et vérifiée à la lecture en
  revue de code ; l'AC12 et le doc-comment disent que la confirmation est une défense en profondeur hors d'atteinte de
  la course.
- **Écartée** : l'ordre (iii) — il monterait à la main un état qu'aucun code de Kesh ne produit (une clôture de N+1
  qui ne tient pas N), et un test qui tue une mutation sur un état fabriqué ne prouve rien du code réel.
- **Réversible** : oui (si `close` cesse un jour de verrouiller ses antérieurs, l'état devient atteignable et un ordre
  (iii) devient le bon test).
- *(Rectifié en validation P3, 2026-10-14, **décision de l'orchestrateur** — P3 R1 = F1, signal D5 au troisième tour
  sur cette famille (P1 F2 / R14 → P2 R5 = F3 → P3 R1 = F1) : « une forme qui produit un défaut à chaque correction se
  retire ». Le raisonnement ci-dessus ne regardait qu'un sens de divergence ; la réouverture de N+1, qui ne prend pas
  N, produit l'autre. **M-b13 est retirée** de la table des mutations, et rien ne la remplace : la fiche ne porte plus
  aucune affirmation d'équivalence ni d'inatteignabilité à son sujet, ni de « défense en profondeur hors d'atteinte ».
  Restent : M-b3, tuée par T-b3 ; la prescription de l'AC11 (« le statut de N+1 n'est jugé que par la confirmation
  verrouillante ») ; et, à l'AC12, la seule phrase que la réouverture de N+1 peut changer l'état lu entre la vue et la
  confirmation, et que c'est la confirmation qui décide. L'écartée « ordre (iii) » et la réversibilité ci-dessus
  sont caduques avec M-b13.)*

### C-29-1-28 — `api-external.md` et CHANGELOG de la 29-1b passent à l'AC17

- **Contexte** : l'AC19 retirée portait aussi ces deux livrables de la 29-1b.
- **Retenu** : l'AC17 les porte (route rejouée et deux codes d'erreur au § 10 de `api-external.md` ; entrée
  `### Ajouté` dans `## [0.14.0] — Non publié`) ; aucune renumérotation des AC (l'AC19 garde son titre en renvoi).
- **Écartées** : une AC20 neuve (renumérotation inutile) ; les répartir entre AC13 et AC14 (le CHANGELOG n'y a pas sa
  place).
- **Réversible** : oui.

### C-29-1-29 — Comptes de l'E2E de parcours créés par l'API à l'étape 0

- **Contexte** : P2 R7 = F4 — le préréglage `with-company-no-fy` ne pose ni compte 1300, ni compte bancaire, et son
  1100 est le compte débiteurs par défaut, refusé comme compte lié.
- **Retenu** : la spec crée `1300 Actifs transitoires` et `1020 Banque` (`Asset`, numéros et noms des plans livrés)
  puis un compte bancaire lié à 1020 ; la charge de l'étape 4 est `4000 Charges CI` ; réserve sur les comptes bancaires
  du préréglage retirée.
- **Écartées** : lier 1000 (accepté, mais compte d'impôt préalable par défaut du préréglage) ; enrichir le préréglage
  (touche `test_fixtures.rs` et tous les tests qui le comptent).
- **Réversible** : oui.

### C-29-1-30 — Fiche d'écriture : la liste des exercices lue à l'ouverture, `startEdit` garde sa propre lecture

- **Contexte** : P3 R7 — le bouton de l'AC15 a besoin de la liste des exercices dès l'ouverture de la page ; le mode
  édition (`startEdit`, `+page.svelte` l. 211-235) la lit déjà, mais au clic, par `Promise.allSettled`, avec sa
  dégradation propre (bornes de date absentes, jamais d'enregistrement bloqué).
- **Retenu** : une lecture neuve à l'ouverture, qui porte ses trois états (chargement, échec, succès) ; `startEdit`
  **inchangé**, relit au clic. Le mock existant de `journal-entry-page.test.ts` (`listFiscalYears: vi.fn()`, l. 49)
  reçoit un comportement par défaut (liste vide résolue) et les trois comportements de T-b8.
- **Écartées** : faire lire à `startEdit` la liste chargée à l'ouverture (elle a pu changer depuis, et son échec
  priverait le mode édition de ses bornes pour une raison qui n'est pas la sienne ; touche un chemin que la story n'a
  pas à toucher).
- **Réversible** : oui.

### C-29-1-31 — Confirmation non verrouillante de N+1 : angle mort écrit, sans mutation ni test

- **Contexte** : P4-4 — M-b13 retirée (C-29-1-27) ; il ne reste que M-b3 (suppression totale). Une confirmation
  rendue non verrouillante n'est tuée par aucune mutation ni aucun test (T-b5 (ii) : A attend déjà à l'étape (1)).
- **Retenu** (décision de l'orchestrateur) : l'écrire comme angle mort à l'AC12, le doc-comment de `reverse_in_tx_inner`
  le disant, et le vérifier en revue de code. Aucune mutation, aucun test neuf.
- **Écartées** : rétablir une mutation sur la confirmation (forme M-b13 : recyclée sur trois tours, signal D5).
- **Réversible** : oui — une mutation pourra être ajoutée si un test de course la tue un jour.

- 2026-10-14 — validation P4 (passe ciblée) remédiée : P4-1 à P4-4, aucune ligne de production touchée ; validation
  close, dev à lancer.

## Choix du développement de la 29-1a (2026-10-14, Opus 5.5, `bmad-dev-story`)

### C-29-1-32 — `balances_from_row` passe `pub(crate)`

- **Contexte** : l'AC3 écrit `fn balances_from_row` (privée) ; or K3, dans `closing_checks`, doit passer d'une
  `BalancesRow` à des `BankAccountBalances` pour appeler `statement_gap` et lire `statement_date`.
- **Retenu** : `pub(crate)`, comme `balances_by_bank_account`, `BalancesRow` et `is_shared`. Le `match` reste écrit une
  seule fois.
- **Écartées** : faire rendre des `BankAccountBalances` à `balances_by_bank_account` (K3 perdrait `is_shared`, qui lit
  la ligne) ; recopier le `match` dans `closing_checks` (DRY).
- **Réversible** : oui.

### C-29-1-33 — `count` de K1 et K7 : 1 en `attention`, 0 sinon

- **Contexte** : la fiche donne un `count` à chaque contrôle ; K1 (antérieur ouvert) et K7 (suivant absent) n'ont pas
  de liste à compter.
- **Retenu** : `count` = nombre de points relevés ; pour K1 et K7, `1` quand le contrôle est en `attention`, `0` sinon.
  Écrit au doc-comment du module et à `api-external.md`.
- **Écartées** : `count` facultatif (forme JSON variable selon le code) ; compter les antérieurs ouverts (le panneau
  n'en nomme qu'un, le plus ancien).
- **Réversible** : oui (changement de contrat à annoncer).

### C-29-1-34 — K4 : libellé = nom du contact, lien par facture et lien filtré

- **Contexte** : « le libellé est celui de la liste des factures » ; un brouillon n'a pas de numéro, la liste l'affiche
  par son contact (`invoiceNumber ?? contactName`).
- **Retenu** : `label` = `contacts.name` ; chaque brouillon mène à `/invoices/{id}` ; le lien du contrôle ouvre
  `/invoices?status=draft&dateFrom=…&dateTo=…`, filtres que la liste lit déjà dans l'URL. `date` ajoutée à chaque
  élément (affichée au format suisse).
- **Réversible** : oui.

### C-29-1-35 — T-a2 : le volet (a) lit aussi le source décommenté

- **Contexte** : l'AC2 retire les lignes de commentaire pour le volet (b) seulement. La doc du module cite `query*` en
  prose.
- **Retenu** : les deux volets lisent le source hors tests **décommenté** (lignes dont le premier non-blanc est `//`) ;
  un site n'est d'ailleurs compté que si le mot est suivi de `::<` ou de `(`. Un appel en commentaire n'est pas du code.
- **Écartée** : le volet (a) sur le source brut — un commentaire citant `query(` rougirait sans défaut.
- **Réversible** : oui.

### C-29-1-36 — K7 : le lien « Créer l'exercice suivant » ouvre le dialogue de création de la même page

- **Contexte** : l'AC5 demande un lien *Nouvel exercice* (Admin / Comptable) ; la création est un dialogue de la page
  des exercices, sans route propre.
- **Retenu** : un bouton du panneau ferme le panneau et ouvre ce dialogue ; absent pour Consultation
  (`onCreateFiscalYear` non fourni).
- **Réversible** : oui.

### C-29-1-37 — Le rappel du dialogue de clôture fait sa propre lecture

- **Contexte** : l'AC6 dit « charge les mêmes contrôles à son ouverture ».
- **Retenu** : une lecture à chaque ouverture du dialogue (jamais la réponse du panneau, qui peut dater) ; un jeton de
  requête écarte une réponse arrivée pour un autre exercice ; le lien « Voir le détail » ferme le dialogue de clôture et
  ouvre le panneau, qui relit.
- **Réversible** : oui.

### C-29-1-38 — K6 en une requête à drapeaux, liste tronquée en Rust

- **Retenu** : une ligne par écriture de la population (écritures de Y, ou datées dans Y), un drapeau `CAST(… AS
  SIGNED)` par motif, `HAVING` sur leur disjonction ; `count` = nombre de lignes (écritures distinctes par
  construction), `countByReason` sommé en Rust, liste tronquée à 20 dans l'ordre `entry_date, id`. Les totaux de
  lignes sont agrégés dans une table dérivée bornée à la société.
- **Écartées** : une requête par motif puis déduplication en Rust (quatre lectures, dédoublonnage à écrire).
- **Réversible** : oui.

### C-29-1-39 — E2E T-a13 sur le préréglage `with-company`, captures « sombre » = clair

- **Retenu** : la spec `fiscal-years.spec.ts` réutilise son préréglage (`with-company`, exercice 2020-2030) et le relevé
  CAMT synthétique `camt053_v04_minimal.xml` réécrit (une transaction en attente, IBAN de test) ; captures du panneau et
  du rappel en `light` et `dark` dans `kesh-gate-logs/29-1a-captures/`. Kesh n'a pas de thème sombre (C-28-1-7) : les
  captures « sombres » sont identiques octet pour octet aux claires (`cmp`), ce que la recette constate.
- **Réversible** : oui.

### C-29-1-40 — Trois formulations reprises pour les gardes de rédaction

- **Contexte** : au gate, G8 / G8-bis (`kesh-i18n`, verbe « rouvrir » en fr-CH sans ordre ni borne), G11 (allemand :
  « Schliessen » nu réservé aux panneaux) et G18 (`textes_coherents`, « autre société » au manuel) ont rougi sur des
  textes neufs.
- **Retenu** : « fermez ce panneau, puis ouvrez-le de nouveau » (fr-CH et repli) ; « Öffnen Sie dieses Fenster später
  erneut » (de-CH) ; au manuel, la limite de K6 dit « le compte de chaque ligne appartient bien au plan comptable du
  dossier de l'écriture » au lieu de « une autre société ». Les gardes ne sont pas touchées.
- **Réversible** : oui.

### C-29-1-41 — `api-external.md` : une ligne « Exercices comptables » et la note ¹⁰

- **Contexte** : le tableau des ressources n'avait pas de ligne d'exercices ; l'AC3 disait « rien à ajouter » pour les
  champs du relevé.
- **Retenu** : la ligne (lectures, écritures ouvertes aux clés, réouverture fermée — `ensure_not_pat`, vérifié au code)
  et la note ¹⁰ (forme, codes, ordre, remarque sur K5) ; la note nomme `statementGap` comme le même écart, en une
  phrase.
- **Réversible** : oui.

### C-29-1-42 — PDF du manuel d'administration et de la brochure non commités

- **Contexte** : `make fr` régénère les trois PDF ; seules les sources du manuel utilisateur ont changé.
- **Retenu** : les deux autres PDF, recompilés sans changement de source, sont restaurés (`git checkout --`) : leur
  différence n'était que de compilation.
- **Réversible** : oui.

### C-29-1-43 — La 29-1a part dans la v0.13.4 (décision de l'orchestrateur, revue de code P1)

- **Contexte** : l'AC9 plaçait l'entrée sous `## [0.14.0] — Non publié` et prévoyait son passage en 0.13.4 si ce tag
  suivait la fusion (F8 de la revue P1). L'orchestrateur a tranché : la 29-1a part dans la **v0.13.4** ; la 29-1b
  reste en v0.14.0.
- **Retenu** : l'entrée (Ajouté, Modifié, Corrigé) passe sous `## [0.13.4] — Non publié`, après celle de la 29-2 dans
  `### Corrigé` ; la note ¹⁰ d'`api-external.md` dit « Depuis la v0.13.4 ». La section `## [0.14.0] — Non publié`
  reste, **vide**, pour la 29-1b — à vérifier avant le tag v0.13.4 (une section vide publiée avec la 0.13.4).
- **Réversible** : oui, déplacement de texte.

### C-29-1-44 — M-a33 (population de K6 élargie) déclarée équivalente

- **Contexte** : la revue P1 (E3) demandait que les écritures incohérentes **hors** de Y ne sortent pas. La mutation
  qui élargit le `WHERE` de population (`… OR TRUE`) **survit** au test neuf.
- **Retenu** : équivalente, démontrée : chacun des quatre drapeaux porte déjà sa condition sur Y (`fiscal_year_id = Y`
  pour trois, `entry_date BETWEEN` pour le quatrième) ; une écriture hors population a ses quatre drapeaux à zéro et
  le `HAVING` l'écarte. Le `WHERE` n'est qu'un filtre de lignes. Écrit au doc-comment d'`INCONSISTENT_SQL`. Ce qui
  tient la promesse « hors de Y, rien ne sort », ce sont les gardes des drapeaux — M-a27 et M-a28, tuées.
- **Réversible** : sans objet (aucun code changé).

### C-29-1-45 — Avoirs en brouillon (B-1) : limite écrite, pas de K4-bis

- **Contexte** : K4 ne lit que `invoices` ; `credit_notes` a un statut `draft` dans sa contrainte.
- **Retenu** : aucune lecture neuve. `credit_notes::create` insère directement au statut `issued` (une seule étape,
  DC5) : aucun chemin de l'application ne produit un avoir `draft`. La limite est écrite au manuel, à la note ¹⁰ et
  au doc du module.
- **Écarté** : un K4-bis (contrôle neuf : changement de contrat de l'API, pour un état inatteignable).
- **Réversible** : oui.

### C-29-1-46 — Formes que T-a2 ne voit pas (B-2 = E LOW-4 = F7) : écrites, garde non élargie

- **Contexte** : `query_with`, macros `query!`, `QueryBuilder`, `raw_sql`, `Executor` sur chaîne, alias, mot de
  verrou coupé par une continuation de chaîne : la doc du module disait « toute requête ».
- **Retenu** (consigne de l'orchestrateur) : la doc du module dit exactement ce que la garde lit et liste ces formes
  comme angle mort vérifié en revue ; la garde n'est pas élargie — sa forme a été réduite à deux volets en validation
  (C-29-1-27), l'élargir rouvrirait ce débat.
- **Réversible** : oui.

### C-29-1-47 — Coût de K6 (B-3 = E LOW-5) : écrit, non borné

- **Retenu** : le doc du module dit que la table dérivée agrège tout le grand livre de la société, à chaque ouverture
  du panneau et du dialogue ; aucune borne ajoutée sans mesure (aucune ligne de production ne change dans cette
  remédiation).
- **Réversible** : oui.

---

## Story 29-1b — Contre-passer au début de l'exercice suivant (développement, 2026-10-14, Opus 5.5)

### C-29-1-48 — L'exercice de l'écriture absent de la liste : motif « indisponible »

- **Contexte** : l'AC15 tranche cinq états de la liste ; un sixième est possible — la liste lue ne contient pas
  l'exercice de l'écriture (état incohérent, ou exercice supprimé entre la fiche et la liste).
- **Retenu** : traité comme une liste illisible (`unavailable`) : motif « La liste des exercices n'a pas pu être lue… »,
  pas de bouton. On ne sait pas ce qui suit un exercice qu'on ne voit pas.
- **Écartées** : « aucun exercice ne suit » (faux : on n'en sait rien) ; ni bouton ni motif (contraire à la règle de
  la page, un bouton absent s'explique).
- **Réversible** : oui.

### C-29-1-49 — Un seul dialogue de confirmation pour les deux contre-passations

- **Contexte** : la page avait un dialogue « Contre-passer » ; le geste neuf en demande un qui nomme date et exercice.
- **Retenu** : un seul dialogue (`reverseConfirm = 'today' | 'nextYear' | null`), son titre, son corps et le
  `data-testid` de son bouton (`reverse-entry-confirm` / `reverse-entry-next-year-confirm`) selon le geste — règle
  DRY du dépôt. Le mode « exercice suivant » ne s'affiche que si l'exercice suivant est ouvert. Sur un refus, le
  geste du jour garde son comportement (toast, dialogue ouvert) ; le geste neuf ferme le dialogue et relit la fiche,
  pour que le motif prenne la place du bouton.
- **Écartées** : un second dialogue recopié (duplication) ; changer le comportement d'erreur du geste du jour (hors
  périmètre).
- **Réversible** : oui.

### C-29-1-50 — Confirmation : l'exercice confirmé doit être celui qu'on a trouvé (`Invariant` sinon)

- **Contexte** : `find_open_covering_date(start_date)` rend l'exercice ouvert qui couvre le premier jour du suivant
  lu ; par non-chevauchement (`find_overlapping`), c'est lui.
- **Retenu** : un écart d'identifiant rend `DbError::Invariant` (défaut structurel, 500), non un refus métier. Ce
  n'est pas un jugement de statut sur la lecture ordinaire (C-29-1-27 tenu) : seul l'identifiant est comparé.
- **Écartées** : aucune garde (un état impossible passerait en silence) ; `NextFiscalYearClosed` (il dirait faux).
- **Réversible** : oui.

### C-29-1-51 — La liste des exercices se lit à l'ouverture de la page, pour tous les rôles, sous jeton

- **Retenu** : `loadFiscalYears()` appelé par `loadEntry` (donc aussi après navigation et après un refus), jeton qui
  écarte une réponse tardive, appel enveloppé par `Promise.resolve().then(…)` et réponse non-tableau traitée en
  échec — un mock ou un client qui rendrait `undefined` donne le motif d'indisponibilité, jamais une exception.
  Lue aussi pour Consultation (rien ne s'affiche : coût d'une lecture, simplicité du chemin).
- **Écartées** : lecture au clic (l'AC15 veut le motif avant le clic) ; lecture réservée aux rôles d'écriture.
- **Réversible** : oui.

### C-29-1-52 — Le jugement de l'exercice suivant, fonction pure de `features/journal-entries/`

- **Retenu** : `next-year-reversal.ts` (`nextYearReversal(years, fiscalYearId)`) : premier exercice dont `startDate`
  est postérieure à la fin de celui de l'écriture (ordre lexical des dates ISO), comme `find_next_after`. Sans i18n :
  la page traduit le motif.
- **Réversible** : oui.

### C-29-1-53 — E2E de parcours sur des années fixes et passées (2025, 2026)

- **Contexte** : l'AC18 dit N et N+1 ; un relevé CAMT figé a des dates.
- **Retenu** : N = 2025, N+1 = 2026, relevé du 01.12.2025 au 31.12.2025 — rien ne dépend du jour d'exécution, le
  fichier se lit tel quel (aucune réécriture d'années).
- **Écartées** : années dérivées de l'horloge (relevé à réécrire, années futures possibles).
- **Réversible** : oui.

### C-29-1-54 — Emplacement des tests

- **Retenu** : tests du dépôt dans un fichier neuf, `crates/kesh-db/tests/reversal_next_fiscal_year.rs` ; garde T-b4
  dans le module de tests de `journal_entries.rs` (elle lit ce source) ; T-b2 et T-b7 **ajoutés** à
  `journal_entry_reversal_e2e.rs` (ses aides `setup`, `consultation_token`), sans modification des tests existants ;
  T-b8 dans `journal-entry-page.test.ts` (Tb4).
- **Réversible** : oui.

### C-29-1-55 — Un sixième compteur du registre des routes, non nommé par la fiche

- **Contexte** : l'AC13 énumérait les sites de `audit_route_registry.rs` à bouger ; le gate a rougi sur un autre,
  `every_replayed_route_calls_an_envelope` : `examinees, 24` et « les vingt-quatre routes Rejouee ».
- **Retenu** : 24 → 25 et « vingt-cinq » ; valeurs `\b(24|118|112|121)\b` regrepées sur le fichier et sur
  `docs/api-external.md` : plus aucun site de compteur.
- **Réversible** : oui.

### C-29-1-56 — Le mock d'`i18nMsg` du test de page remplace les variables

- **Contexte** : le mock rendait le repli brut ; la confirmation doit nommer date et exercice (T-b8).
- **Retenu** : les `{ $x }` du repli sont remplacés par les arguments, comme le fait `i18nMsg` ; les cas existants
  restent verts (aucun ne dépendait du texte brut).
- **Réversible** : oui.

### C-29-1-57 — Textes du geste : une aide dans le dialogue, le bouton de confirmation réutilisé

- **Retenu** : clés `journal-entries-reverse-next-year-{action, dialog-title, dialog-body, help,
  reason-unavailable, reason-missing, reason-closed}` (sept, ×4 locales) ; le « libellé d'aide » de l'AC16 est le
  second paragraphe du dialogue (à quoi sert le geste, ce qui se fige) ; le bouton de confirmation et « Annuler »
  reprennent `journal-entries-reverse-confirm` / `-cancel`, et le succès `journal-entries-reverse-success`. Le motif
  « clôturé » de l'écran et le refus serveur `error-next-fiscal-year-closed` ont la même phrase fr-CH, deux clés
  (l'une d'écran, l'autre du serveur, comme `journal-entries-modify-blocked-later-fiscal-year-closed` /
  `error-later-fiscal-year-closed`).
- **Réversible** : oui.

### C-29-1-58 — Brochure : deux points au lieu d'une phrase « à venir »

- **Retenu** : le point « report automatique » ne dit plus « reste à venir », il dit qu'aucune écriture de clôture
  n'est à passer (le résultat est calculé) ; un point neuf nomme la préparation de la clôture et les transitoires.
  Les écritures de clôture ne sont pas annoncées (écartées par conception, C-29-1-2).
- **Réversible** : oui.

## Remédiation de la revue de code P1 de la 29-1b (2026-10-14)

### C-29-1-59 — La confirmation du geste neuf nomme l'écriture ; celle du jour garde son texte

- **Contexte** : B-1 (revue P1) — une confirmation qui ne nomme que la date et l'exercice ne dit pas *quelle* écriture
  sera contre-passée ; décision de l'orchestrateur : numéro et libellé en plus.
- **Retenu** : `journal-entries-reverse-next-year-dialog-body` porte `{ $number }` et `{ $description }` (×4 locales,
  repli Svelte égal à fr-CH). Le dialogue du geste **du jour** (Story 24-4a) n'est pas touché : sa clé est antérieure à
  la story, et le défaut de navigation qui le rendait dangereux est fermé par C-29-1-60.
- **Écarté** : nommer l'écriture dans les deux dialogues (élargit la story à un texte de la 24-4a).
- **Réversible** : oui.

### C-29-1-60 — `loadEntry` ferme toutes les confirmations, suppression comprise

- **Contexte** : B-1 — `reverseConfirm` survivait à une navigation (composant réutilisé), et la confirmation se
  rouvrait sur l'écriture suivante. `showDeleteConfirm` avait le même défaut (Story 15-8b).
- **Retenu** : `reverseConfirm = null` et `showDeleteConfirm = false` en tête de `loadEntry`. Le `reverseConfirm = null`
  du chemin de refus est gardé (il dit l'intention) bien que `loadEntry` le fasse aussi, de façon synchrone : M-b18
  est donc équivalente, démontrée (cf. Dev Agent Record).
- **Réversible** : oui.

### C-29-1-61 — Un `page` réactif de test, dans un module `.svelte.ts` voisin du test

- **Contexte** : tester une navigation (B-1, E-3) exige que `page.params.id` change et que le `$effect` de la page se
  rejoue ; un objet simple dans `vi.mock` ne déclenche rien, et les runes ne vivent que dans un `.svelte.ts`.
- **Retenu** : `page-state.test-helper.svelte.ts` (un `$state` exporté, muté et jamais réassigné), importé par le seul
  `journal-entry-page.test.ts` ; `afterEach` remet l'`id` à `7`. SvelteKit ignore ce fichier (pas de préfixe `+`).
- **Écarté** : un helper partagé sous `src/lib/` (aucun autre test n'en a besoin aujourd'hui).
- **Réversible** : oui.

### C-29-1-62 — B-2 et B-3 écrits comme limites, sans changement d'écran

- **Contexte** : B-2 (refus `PERIOD_LOCKED` : le bouton revient sans motif, le refus se répète), B-3 (le motif
  « créez d'abord l'exercice suivant » s'affiche presque toute l'année). Décision de l'orchestrateur : limites écrites.
- **Retenu** : le manuel (§ *Les transitoires*) dit les deux ; le commentaire du chemin de refus de la page dit B-2.
- **Écarté** : lire `booksLockedThrough` à l'ouverture (sixième motif, étend l'AC15) ; restreindre le motif aux
  écritures de fin d'exercice (règle de la page : un bouton absent s'explique).
- **Réversible** : oui — les deux améliorations restent possibles.

### C-29-1-63 — Brochure : un bloc propre à l'Epic 29, « en cours — à paraître »

- **Contexte** : M1 (revue P1, lentille A) — le point neuf de C-29-1-58 était rangé sous « Epic 14, v0.8, livré », et
  disait « d'un clic » pour un geste qui en demande deux.
- **Retenu** : le point est retiré du bloc de l'Epic 14 ; un bloc « Préparer la première clôture (Epic 29, en cours —
  à paraître) », icône d'attente (`\faClock`, couleur `keshWarning`, comme le *Backlog*), porte deux points marqués
  « à paraître en v0.13.4 » (contrôles préalables) et « à paraître en v0.14.0 » (transitoires, « depuis la fiche de
  l'écriture »). Annote C-29-1-58, dont le premier volet (le point « report automatique » sans « à venir ») reste.
- **Réversible** : oui ; à la release, le bloc passera en `keshSuccess` / « livré ».

### C-29-1-64 — README et site : l'Epic 29 « en cours », rien de promis comme publié

- **Contexte** : E-7 (`website/roadmap.html:43` annonçait comme manquantes des « écritures de clôture » écartées par
  conception) ; L6 (pas de ligne Epic 29 à la feuille de route du README) ; et la section *Fonctionnalités* du README
  marquait ✓ deux fonctions qu'aucune image publiée ne porte.
- **Retenu** : README — ligne « v0.13.4 · v0.14.0 (en cours) | E29 … | 🚧 En cours », et les deux ✓ remplacés par
  « *(à paraître en v0.13.4)* » / « *(à paraître en v0.14.0)* » ; site — phrase de l'en-tête corrigée (« la préparation
  de la clôture (#232, epic 29, en cours, non publiée) »), carte E29 « In progress — not yet released ».
- **Réversible** : oui ; à la release, les marques « à paraître » tombent.

## Report de la 29-1b sur la v0.13.4 (2026-10-15, Opus 5.5)

### C-29-1-65 — CHANGELOG : la section `## [0.14.0] — Non publié` au-dessus de `[0.13.4]` datée

- **Contexte** : la 29-1b écrivait ses entrées sous `## [0.14.0] — Non publié` puis `## [0.13.4] — Non publié` ; la
  release a daté `[0.13.4] — 2026-10-15`. Les entrées de la 29-1b ne doivent pas rester dans une version publiée.
- **Retenu** : la forme `## [X.Y.Z] — Non publié`, la seule que `scripts/prepare-release.sh` (pré-vol (0a), lignes
  237-261) reconnaît et date ; une `## [Unreleased]` ferait échouer son pré-vol. Section `[0.14.0]` de la branche,
  puis la section `[0.13.4]` datée de main, intacte ; l'en-tête `[0.13.4] — Non publié` de la branche est retiré.
- **Écarté** : `## [Unreleased]` (forme Keep a Changelog, inconnue du script).
- **Réversible** : oui.

### C-29-1-66 — Registre d'audit des routes : 120 / 114 / 123, union des deux ajouts

- **Contexte** : la 29-8 (rattachement) et la 29-1b ajoutaient chacune une route à `lib.rs` ; les deux côtés
  écrivaient 119 / 113 / 122.
- **Retenu** : 120 routes d'inventaire, 114 tracées, 123 au registre (`tout.len()` compris), rejouées 25 (24 de main
  plus la 29-1b ; le rattachement est `SansEcritureAuJournal`, 95 inchangé) ; en-tête du module aligné (120 / 123).
  Le test recompte depuis la source, il juge.
- **Réversible** : oui.

### C-29-1-67 — Compteur `sitesTotal` : 2186 + 7 = 2193

- **Contexte** : `i18n-keys.test.ts` — main en 2186 (29-6, 29-8, 29-9), la branche écrivait 2163 → 2170.
- **Retenu** : l'apport de la 29-1b (+7, `+page.svelte` de la fiche d'écriture) ajouté au total de main, la
  généalogie de la branche gardée, annotée « écrit 2163 → 2170 sur sa base ». La collecte du test juge.
- **Réversible** : oui.

### C-29-1-68 — README, site, brochure : ce qui est publié en v0.13.4, ce qui paraît en v0.14.0

- **Contexte** : C-29-1-63 et C-29-1-64 prévoyaient que les marques « à paraître » tomberaient à la release ; la
  v0.13.4 a publié les contrôles préalables (29-1a), pas les transitoires.
- **Retenu** : README — la ligne de feuille de route `v0.13.4 (correctif)` de main (✅ Publiée) est gardée, une ligne
  `v0.14.0 (en cours)` (« seconde partie » : les transitoires ; la 29-7 dite « à écrire ») est ajoutée en 🚧 ; dans *Fonctionnalités*,
  le texte de main pour les contrôles (✓) et les transitoires marquées « *(à paraître en v0.14.0)* ». Site — la carte
  E29 « part one » publiée de main gardée, une carte « part two, v0.14.0, In progress — not yet released » ajoutée ;
  l'en-tête dit les contrôles livrés en v0.13.4 et la contre-passation à venir en v0.14.0. Brochure — le bloc
  Epic 29 reste « en cours » (`keshWarning`), les contrôles « livré en v0.13.4 », les transitoires « à paraître en
  v0.14.0 ».
- **Écarté** : passer le bloc de la brochure en `keshSuccess` (l'epic reste ouvert).
- **Réversible** : oui ; à la v0.14.0, les marques « à paraître » tombent.

### C-29-1-69 — LOW-1 et LOW-2 de la passe ciblée P2 : texte seul

- **Retenu** : LOW-1 — le doc-comment de `NextFiscalYearClosed` nomme les deux suppressions d'exercice (remise à zéro,
  import d'installation, `backup::TABLES_TO_TRUNCATE` enfants vers parents). LOW-2 — le manuel (§ *Les transitoires*)
  et le commentaire du chemin de refus de la page citent les deux refus qui laissent le bouton offert
  (`PERIOD_LOCKED`, `LATER_FISCAL_YEAR_CLOSED`). LOW-3 (tests manquants) et LOW-4 (`break-words`, une ligne de
  production) laissés, comme le rapport le propose.
- **Réversible** : oui.

## Story 29-5 — À score égal, la facture échue la première passe en tête ([#668])

### C-29-5-1 — La story entre dans l'Epic 29 et sort avec la 29-2

- **Contexte** : #668 est relevée pendant les fiches 29-2 et 29-3 (C-29-2-7, C-29-2-11, C-29-3-12), hors de leur
  périmètre minimal. La 29-2 élargit la fenêtre (candidates jusqu'à `F ≥ D − 60` à 30 jours net, factures non échues
  à échéance lointaine) et rend le départage par `invoice_id` plus coûteux.
- **Retenu** : story ajoutée à l'Epic 29 par l'orchestrateur le 2026-10-14, après la 29-3, dans la même v0.13.4
  (mergée avant le tag, C-29-2-13). Indépendante de la 29-4.
- **Écartée** : la reporter après la v0.13.4 (la fenêtre élargie sortirait sans le départage qui la rend sûre).
- **Réversible** : oui.

### C-29-5-2 — Le départage suit l'art. 87 CO : date d'exigibilité, puis date, puis identifiant

- **Contexte** : l'attendu d'origine de #668 (« la récente passe en tête », proximité de la date d'ancrage) contredit
  l'art. 87 CO, comme le relève la rectification de Guy ; l'attendu révisé : d'abord les échues, puis l'ancienneté.
- **Retenu** : `X = max(F, E)` (échéance absente valant la date, art. 75 CO), clé `(X, F, id)` croissante après le
  score. Constat : comme toute échue a une `X` inférieure à toute non échue, ce tri place les échues d'abord **sans**
  lire la date du paiement, puis celle échue la première.
- **Écartées** : (a) proximité de l'ancre au paiement — contraire à l'art. 87, et l'ancre mêle les rappels ;
  (b) clé explicite `(non échue, X, …)` dépendant de `D` — équivalente, plus coûteuse (un paramètre de plus au SQL) ;
  (c) la date de facture seule — ignore l'échéance, c'est le défaut.
- **Réversible** : oui (une fonction, une expression SQL).

### C-29-5-3 — Un rappel ne vaut pas poursuite ; le critère des premières poursuites reste sans donnée

- **Contexte** : la consigne suggère « premières poursuites (en Kesh : rappels ?) ». Kesh n'enregistre aucune
  poursuite (`grep -rni poursuite` : le seul libellé « Dernier rappel avant poursuite »). Un rappel est une
  interpellation (art. 102 CO), une poursuite une réquisition au sens de la LP.
- **Retenu** : ne pas lire les rappels pour le départage ; écrire l'angle mort (manuel, doc-comment) avec le recours :
  enregistrer le règlement depuis la fiche de la facture voulue, puis rejeter la transaction.
- **Écartée** : les rappels comme substitut — juridiquement faux, et il faudrait les charger pour chaque candidate
  (`propose_matches` n'en a pas la donnée).
- **Réversible** : oui (si Kesh enregistre un jour les poursuites).

### C-29-5-4 — L'`ORDER BY` du `LIMIT 50` suit l'ordre d'imputation

- **Contexte** : `ORDER BY i.date ASC, i.id ASC LIMIT 50` choisit les candidates scorées ; avec la fenêtre de la
  29-2, des factures anciennes non échues peuvent pousser hors des 50 une facture échue (C-29-2-7, renvoyé ici ;
  complément de l'orchestrateur du 2026-10-14).
- **Retenu** : `ORDER BY {X} ASC, i.date ASC, i.id ASC`. Les deux ordres coïncidant, dans chaque classe de score les
  candidates gardées forment un préfixe de l'ordre d'imputation : la troncature n'écarte jamais celle que le
  départage placerait en tête. La troncature reste aveugle au score (C-28-7-10, inchangé, écrit).
- **Écartées** : (a) la troncature écrite comme simple limite — le correctif tient en une ligne ; (b) trier par score
  en SQL — dupliquerait le score en SQL (C-28-7-10, exclusion de la 28-7).
- **Réversible** : oui.

### C-29-5-5 — Al. 2 et al. 3 : ordres par défaut écrits

- **Contexte** : échéances identiques → imputation proportionnelle (al. 2) ; aucune échue → la moins garantie
  (al. 3). Kesh propose une facture et ignore les garanties.
- **Retenu** : à `X` égale, la plus ancienne par date puis par identifiant ; parmi les non échues, `X` croissante.
  Écrit au manuel (recours : règlements depuis les fiches) et au doc-comment, avec la condition qui fonde
  l'équivalence de C-29-5-2.
- **Écartée** : proposer plusieurs factures, ou une répartition — un écran et un modèle de proposition neufs, hors
  périmètre.
- **Réversible** : oui.

### C-29-5-6 — Le score reste la première clé

- **Contexte** : une désignation par le débiteur (référence, art. 86) prime l'imputation légale ; elle passe par le
  score de référence.
- **Retenu** : le score n'est pas touché ; l'ordre d'imputation ne départage que des scores égaux. Une facture **du même client** au
  montant exact non échue passe devant une échue à quelques centimes près (score) — **même désignée par le numéro**
  (le montant pèse 0,50, la référence 0,40, `matching.rs:80` ; validation P1, R2 : la désignation ne prime qu'à
  montant égal, ce que le manuel dit). Testé (AC5 cas 8, M7).
  *Amendé à la validation P2 (R1, F1, R3 ; 2026-10-14)* : la phrase précédente, écrite sans condition de client,
  est bornée au **même client** ; une facture d'un **autre** client au montant exact **peut** passer devant une facture désignée à quelques
  centimes près, non toujours — 0,40 + 0,10 (désignée, contact au nom du client) contre 0,50 (montant seul) sont
  égaux, et l'ordre d'imputation tranche alors. Et la désignation au montant exact passe devant toute facture que la
  référence ne désigne pas **aussi**, comme devant une facture désignée à montant différent ; entre deux factures
  désignées au montant exact, l'ordre d'imputation tranche (`reference_score` est un contenant bidirectionnel).
  *(Rectifié le 2026-10-14, passe ciblée P3, P3-1 : la phrase disait « ne passe devant que », trop étroit.)* Manuel, § *Ce que la
  règle ne fait pas* et doc-comment écrits sur ce modèle (« peut passer devant », « toujours si elle est du même
  client »).
- **Écartée** : exigibilité avant score — ferait passer une facture échue devant la facture que le paiement nomme.
- **Réversible** : oui.

### C-29-5-7 — Aucun texte d'écran ; le manuel dit la règle et ses limites

- **Contexte** : l'écran affiche une candidate ; la phrase d'aide de la 29-3 (« la facture la mieux notée ») reste
  vraie. Entre clients différents à score égal, l'art. 87 est sans objet.
- **Retenu** : ni clé i18n ni capture ni spec Playwright neuve ; la frontière HTTP est exercée par un test
  `spawn_app` (AC6) ; le manuel FR dit la règle, la désignation qui prime, les poursuites et l'al. 2 absents, et
  l'ordre par défaut entre clients ; E2E complet au dernier commit de code.
- **Écartée** : afficher l'échéance ou la raison du choix à l'écran — demande d'évolution, sans demandeur.
- **Réversible** : oui.

### C-29-5-8 — Deux écritures (Rust, SQL) dans le module de la fenêtre, tenues par un test de parité

- **Contexte** : `propose_matches` (Rust, pur) et l'`ORDER BY` (SQL) doivent appliquer la même `X`. Précédents :
  `payment_date` et `InvoiceDateWindow` (fonctions pures du dépôt, lues par `kesh-api`).
- **Retenu** : `invoice_exigibility_date`, `imputation_order` et `INVOICE_EXIGIBILITY_DATE_SQL` dans
  `kesh-db/src/repositories/reconciliation.rs` ; `kesh-reconciliation` les appelle (il dépend déjà de `kesh-db`) ;
  test de parité SQL ↔ Rust avec valeurs attendues écrites en dur.
  *Précisé à la validation P2 (R4, F2 ; 2026-10-14)* : les fonctions à côté de `payment_date` ; la macro
  `invoice_exigibility_date_sql!` définie **avant** la macro d'ancre de la 29-2 (portée textuelle des `macro_rules!`).
- **Écartées** : (a) méthode sur l'entité `Invoice` — éloigne la fonction de l'expression SQL qu'elle double ;
  (b) rendre `X` dans la requête et l'ajouter au triplet candidat — change la signature de `propose_matches` et ses
  deux appelants.
- **Réversible** : oui.

### C-29-5-9 — L'ancre de la 29-2 est récrite en composant l'exigibilité

- **Contexte** : la fiche de la 29-2 prévoyait `GREATEST(i.date, COALESCE(i.due_date, i.date), …rappel…)` ; la 29-5
  écrit `GREATEST(i.date, COALESCE(i.due_date, i.date))`. Règle DRY du `CLAUDE.md`. *Rectifié à la validation P2
  (R4)* : la forme **commitée** de la 29-2 (`8f483388`) est en deux termes, `CAST(GREATEST(GREATEST(i.date,
  COALESCE(i.due_date, i.date)), COALESCE(DATE(rmd.last_sent_at), i.date)) AS DATE)`, le premier réservé à une
  expression d'exigibilité commune (C-29-2-15).
- **Retenu** : l'ancre compose l'expression d'exigibilité ; équivalente (`GREATEST` associatif), tests de la 29-2
  verts sans modification d'assertion *(sauf l'exception de l'inventaire (c) : une assertion d'ordre que la 29-5 change
  est réécrite et déclarée — passe ciblée P3, P3-6)*.
  *Amendé à la validation P1 (F1, 2026-10-14)* : la composition passe par `concat!` — une macro d'exigibilité
  insérée dans la macro d'ancre de la 29-2 (C-29-5-11) ; aucun test de la 29-2 ne change. Si la forme mergée de la
  29-2 diffère, relecture à T0.
- **Écartée** : laisser deux écritures du même sous-terme.
- **Réversible** : oui.

### C-29-5-10 — Les tests d'ordre existants gardent leur nom

- **Contexte** : `find_unpaid_invoices_orders_by_date_then_id_before_limit` porte des factures sans échéance ; l'ordre
  d'exigibilité s'y réduit à `(date, id)`.
- **Retenu** : nom et assertions inchangés, doc-comment complété ; un test neuf porte l'ordre avec échéances.
- **Écartée** : renommer — bruit de diff sans gain.
- **Réversible** : oui.

### C-29-5-11 — L'expression d'exigibilité s'écrit par `macro_rules!`, insérée dans la macro d'ancre de la 29-2

- **Contexte** (validation P1, F1) : la composition prescrite par C-29-5-9 est impossible entre deux `const &str` —
  `concat!` n'accepte que des littéraux, et le dépôt n'a pas `const_format`. Le dépôt a le précédent exact :
  `columns!` (`repositories/accounts.rs:20-34`), `invoice_columns!` (`repositories/invoices.rs:50-65`),
  `line_ttc_sql!` appelé dans un `concat!` (`:186-216`). L'orchestrateur a donné à l'agent de développement de la
  29-2 la consigne d'écrire l'ancre par `macro_rules!` (2026-10-14).
- **Retenu** : `macro_rules! invoice_exigibility_date_sql` rendant le littéral
  `CAST(GREATEST(i.date, COALESCE(i.due_date, i.date)) AS DATE)`, plus `pub const INVOICE_EXIGIBILITY_DATE_SQL` qui en
  dérive pour `format!` ; la macro d'ancre de la 29-2 l'insère par `concat!` à la place du premier terme de son
  `GREATEST` (C-29-2-15). Tests de la 29-2 inchangés.
  *Amendé à la validation P2 (R4, F2 ; 2026-10-14)* : « à la place des deux premiers arguments » est devenu « à la
  place du premier terme » — la forme commitée de la 29-2 (`8f483388`) a isolé ce terme (C-29-2-15) ; la macro d'exigibilité est définie **avant** la
  macro d'ancre, faute de quoi celle-ci ne la trouverait pas à son développement. Si la forme mergée diffère, relecture à T0, sans changer la nature
  de l'ancre d'une façon qui toucherait ses tests.
- **Écartées** : (a) ancre devenue `fn -> String` ou `LazyLock<String>` — change sa nature, coûte une allocation, et
  sort du patron du dépôt ; (b) ajouter `const_format` — une dépendance pour une concaténation ; (c) deux écritures
  tenues par un test de parité — admise seulement en repli si la composition est impossible sur la forme mergée.
- **Réversible** : oui.

### C-29-5-12 — Une facture sans échéance est tenue pour échue à sa date : convention écrite

- **Contexte** (validation P1, F5) : l'art. 87 al. 1 parle de dette **exigible**, les al. 2 et 3 de dettes **échues**.
  Pour une facture à échéance, les deux coïncident. Pour une facture sans terme, l'art. 75 la rend exigible dès sa
  date (texte non relu à la source) ; dire qu'elle est aussi **échue** à cette date est une assimilation, non une
  lecture du texte.
- **Retenu** : `X = F` vaut pour les deux notions ; la convention est écrite comme telle au doc-comment et dans la
  fiche (§ *Le critère légal*), et le manuel dit « tenue pour due dès sa date ». Conséquence : l'al. 3 (« aucune n'est
  échue ») ne vise jamais, en Kesh, une facture sans échéance datée au plus tard du jour du paiement.
- **Écartée** : traiter la facture sans échéance comme non échue (al. 3) — elle passerait derrière des factures
  échues plus récentes, contre l'art. 75, et l'ordre redépendrait de la date du paiement.
- **Réversible** : oui.

### C-29-5-13 — `api-external.md` : la forme mergée de la 29-3 n'a pas la parenthèse prévue (développement, T0)

- **Contexte** : la fiche prescrivait de remplacer la parenthèse « (identifiant croissant au sol ; à relire si la
  29-5, #668, est mergée au rebase) » ; la phrase mergée de la 29-3 (`docs/api-external.md:549`) porte à la place
  « par score décroissant, puis par identifiant de facture croissant à score égal ».
- **Retenu** : remplacer ce seul segment « puis par identifiant de facture croissant à score égal » par la parenthèse
  prescrite (date d'exigibilité, `invoiceDate`, échéance absente de la réponse, puis date, puis identifiant ; avant la
  v0.13.4 : identifiant seul) ; la parenthèse « l'ordre lui-même est celui des versions précédentes » remplacée
  comme prescrit ; le reste de la phrase de la 29-3 conservé mot pour mot.
- **Écartée** : ajouter la parenthèse en laissant « identifiant croissant à score égal » — deux ordres contradictoires.
- **Réversible** : oui.

### C-29-5-14 — Manuel : un paragraphe titré et un label neuf pour la règle

- **Contexte** : AC7 laisse le placement au choix rédactionnel (« après » la phrase du score nul, ou paragraphe
  propre).
- **Retenu** : la phrase « Une candidate dont le score vaut zéro… » reste seule ; suivent un `\paragraph{À score égal,
  la facture échue la première.}` (`\label{sec:imputation-score-egal}`), un paragraphe de limites (« Cette règle ne
  départage que des scores égaux. … »), puis le paragraphe de la règle d'affectation, inchangé. La phrase de `:2048`
  renvoie à ce label (« à score égal, celle qu'impute la règle de la loi (§…) »). Le `\paragraph` n'étant pas
  numéroté, le renvoi rend le numéro de la sous-section *Algorithme de matching* (§14.1.1 au PDF).
- **Écartée** : prolonger le paragraphe existant — il mêlait déjà score nul et règle d'affectation.
- **Réversible** : oui.

### C-29-5-15 — `propose_matches` trie des paires `(&Invoice, MatchProposal)`

- **Contexte** : AC2 laisse au dev la façon de garder la facture à côté de la proposition le temps du tri.
- **Retenu** : une référence `&Invoice` empruntée au tableau des candidates, sans clone ; `sort_by` (stable), puis
  `map` vers `MatchProposal`. Signature et forme JSON inchangées. `kesh-reconciliation` dépendait déjà de `kesh-db`.
- **Écartées** : indices dans le tableau (moins lisible) ; clone des factures (coût inutile, smoke de 500 000
  scores).
- **Réversible** : oui.

### C-29-5-16 — `make fr` régénère trois PDF ; seul celui du manuel utilisateur est commité

- **Contexte** : `make fr` reconstruit aussi le manuel d'administration et la brochure, dont les sources n'ont pas
  changé ; leurs PDF ne diffèrent que par la date de génération.
- **Retenu** : `git checkout -- docs/manual/fr/admin-manual.pdf docs/manual/fr/marketing-brochure.pdf` dans le
  worktree ; seul `user-manual.pdf` est commité.
- **Écartée** : commiter trois PDF dont deux sans changement de contenu — bruit de diff et de revue.
- **Réversible** : oui.

### C-29-5-17 — Le doc-comment d'`order_candidates` déclare résolu l'angle mort C-29-3-12

- **Contexte** : la 29-3 y écrivait deux angles morts, dont « à score égal, la facture de plus petit identifiant en
  tête, sans égard à l'exigibilité (#668, Story 29-5) ».
- **Retenu** : la description du tri hérité devient « score décroissant, puis, à score égal, l'ordre d'imputation
  (art. 87 CO) » ; l'angle mort est dit **résolu par la Story 29-5**, l'autre (#665) reste. Aucune ligne de code de
  `routes/reconciliation.rs` ne change.
- **Réversible** : oui.

[#668]: https://github.com/guycorbaz/kesh/issues/668

---

## Story 29-4 — Une saisie concurrente ne passe plus sur un compte qu'on clôt

### C-29-4-1 — Fermer la fenêtre par une revérification verrouillante partagée, complétée du rejeu

- **Contexte** : L1 de la 28-4 (#651). La garde non verrouillante lit la vue de l'écrivain ; la clôture ne prend que
  sa propre ligne en exclusif. Sans conflit entre les deux, rien ne les sérialise. La 28-4 a écarté une garde
  verrouillante (exclusive, posée tôt) pour ses cycles (P1 F1/F4, P2 R1/F1/F2/F5), puis a déplacé la vue en la rendant
  non verrouillante à la même place (P3 R1).
- **Retenu** : piste (c) — une lecture **verrouillante partagée** de `closed_on` (`LOCK IN SHARE MODE`, clé primaire),
  `bank_accounts::recheck_open_locked_in_tx`, sur les sept chemins à fenêtre réelle ; complétée de (b), le rejeu de la
  clôture et de la réouverture (C-29-4-5). Les gardes non verrouillantes de la 28-4 restent (C-29-4-3).
- **Écartées** :
  - (a) verrou commun dans un ordre global unique : le dépôt n'en a pas (`journal_entries.rs:1358`) ; l'ordre
    « document → compte → exercice » ferme un cycle avec `POST`, `PUT`, `DELETE /bank-accounts`, non rejouées ;
  - (b) seule : pas de verrou commun, donc pas de cycle à rejouer — la fenêtre n'est pas un interblocage ;
  - (d1) verrou nommé (`GET_LOCK`) pris aussi par la clôture : `with_account_lock` le relâche avant le commit de
    l'appelant (`mutex.rs:131-136`), et un cycle verrou nommé / verrou de ligne n'est pas détecté (`1205`) ;
  - (d2) la clôture verrouille les lignes enfants : ensemble de verrous large, analyse que la 28-4 a dû abandonner ;
  - (d3) refuser la clôture pendant une écriture, ou documenter la fenêtre : inutile, (c) la ferme ;
  - verrou **exclusif** de l'écrivain : rouvre les cycles écrivain / écrivain de la 28-4 (W9 / W24, W30 / W24).
- **Réversible** : oui — retirer sept appels et deux enveloppes rétablit la 28-4.
- *Annotation du 2026-10-14 (remédiation de la validation P1)* : le complément (b) ne porte plus que sur la **clôture**
  (C-29-4-5 amendé) ; s'y ajoute l'épinglage d'`innodb_snapshot_isolation` (C-29-4-13). Réversible : sept appels, une
  enveloppe, un réglage de session.
- *Annotation du 2026-10-14 (remédiation de la validation P2 ; décisions de l'orchestrateur)* : le complément (b) porte
  désormais sur la **clôture, la réouverture et le `PATCH` du compte** (C-29-4-5, C-29-4-14 renversé, C-29-4-19) ;
  l'épinglage d'`innodb_snapshot_isolation` est **retiré** (C-29-4-13, #669) ; la revérification a deux variantes
  (C-29-4-20). Réversible : sept appels, deux fonctions, trois enveloppes.

### C-29-4-2 — Positions : après l'`INSERT` à clé étrangère (W17, W22) ; en dernière acquisition de verrou (annulations)

- **Contexte** : la position d'un verrou neuf décide de ses cycles ; celle d'une lecture non verrouillante, de la vue.
- **Retenu** : W22 juste après `INSERT INTO bank_imports`, W17 juste après `INSERT INTO payment_batches` — le verrou
  partagé du compte y est **déjà tenu** (clé étrangère ; T-a31 le vérifie), la demande ne crée rien ; W9, W24, W25,
  W28, W29 **après leur dernière écriture**, avant l'audit — le compte vient après `companies` (ordre des routes de
  gestion, aucun cycle avec elles) et après l'exercice (cycle avec W8 / W27, rejoués des deux côtés).
- **Écartées** : juste après le verrou du document (cycle avec les routes de gestion non rejouées — R1 de la P2 de la
  28-4) ; au point de la garde de la 28-4 (entre l'exercice et `companies` : même cycle) ; W22 avant l'`INSERT` (verrou
  explicite à une position non analysée, sans gain).
- **Réversible** : oui (une ligne par site).

### C-29-4-3 — Garder les gardes non verrouillantes de la 28-4

- **Contexte** : la revérification seule suffirait à refuser.
- **Retenu** : les garder — refus précoce sans écrire, et ordre des refus fixé par la 28-4 (W24 `HasReconciled` avant
  « clos », motifs d'exercice après la garde, T-a27) ; la revérification ne répond que dans la course.
- **Écartée** : les remplacer par la revérification (l'ordre des refus changerait : un compte clos dans un exercice
  clos rendrait le motif d'exercice au lieu de `BANK_ACCOUNT_CLOSED`).
- **Réversible** : oui.

### C-29-4-4 — Prémisses de #651 corrigées : W30 sans fenêtre, W24 seulement pour un import vide

- **Contexte** : #651 liste huit chemins, dont l'annulation de rejet et la suppression d'import.
- **Retenu** : W30 n'a **pas** de fenêtre (I1 : une rejetée est `pending`, la clôture refuse tant qu'elle existe ; le
  test de la 28-4 ne monte la course qu'en posant `closed_on` par SQL direct) ; W24 n'en a une que pour un **import
  vide** (sinon ses transactions `pending` font refuser la clôture). W24 reçoit quand même la revérification ; W30 non.
  À commenter sur l'issue par l'orchestrateur.
- **Réversible** : sans objet (constat).

### C-29-4-5 — Rejouer la clôture et la réouverture

- **Contexte** : en attente de sa ligne, la clôture ne tient rien, mais sa demande exclusive fait attendre derrière elle
  les demandes partagées ultérieures : elle peut fermer un cycle à trois, dont elle est la victime (la plus légère).
- **Retenu** : `retry_app_on_deadlock("bank_accounts::close" / "::reopen")` autour d'une fonction « une tentative » ;
  point (vi) d'`audit_route_registry.rs` passe de sept à neuf routes ; un test à trois connexions (T-a4).
- **Écartée** : ne rien faire (un `500` rare sur la clôture) ; rejouer aussi les routes de gestion (aucun cycle neuf ne
  les atteint, C-29-4-2).
- **Réversible** : oui.
- *Amendé le 2026-10-14 (validation P1, F5)* : **la réouverture n'est pas rejouée**. Pour qu'elle soit victime d'un cycle
  à trois, il faudrait un détenteur partagé de la ligne d'un compte **clos** en attente d'une ressource ; toute écriture
  sur un clos est refusée avant de prendre ce partagé (garde non verrouillante ou lecture sous `FOR UPDATE`), et les
  gestes permis sur un clos (W21, W30) ne touchent pas la ligne du compte. Point (vi) : sept → **huit** (non neuf) ; le
  test T29a-4 ne porte que sur la clôture. Écartée : garder les deux comme défense en profondeur (élargit le périmètre
  minimal sans cas à défendre).
- *Amendé le 2026-10-14 (validation P2, F3 = R9 ; décision de l'orchestrateur)* : l'amendement de la P1 est **renversé**
  — la réouverture est rejouée (C-29-4-14). Point (vi) : sept → **dix** (clôture, réouverture, `PATCH`).

### C-29-4-6 — Découper en 29-4a (import, lot, rejeu) et 29-4b (annulations, textes)

- **Contexte** : plus de cinq modules au total ; le risque n'est pas réparti également.
- **Retenu** : 29-4a sans verrou neuf d'écrivain (quatre modules) ; 29-4b porte le verrou neuf, son cycle et les textes
  (cinq modules, la borne). Une fiche, deux clés (précédent C-28-4-1) ; `refs #651` puis `closes #651` ; aucune release
  entre les deux (la v0.13.4 attend la 29-4b).
- **Écartées** : une seule story (dérogation, comme la 28-4a — non nécessaire ici, la moitié a est cohérente seule :
  l'inventaire de l'AC5 dit `Ouverte("29-4b")` pour les cinq sites restants) ; trois stories (le rejeu seul n'a pas
  d'objet sans la 29-4b ; il est rangé avec la fonction de revérification, même module).
- **Réversible** : oui.
- *Annotation du 2026-10-14 (validation P1, R6 = F7)* : « cinq modules, la borne » était un **sous-décompte** — la 29-4b
  touche aussi des doc-comments de `bank_accounts.rs` et de `routes/reconciliation.rs`. Recompté (fiche, § *Périmètre
  minimal*) : 29-4a cinq modules de code (`pool` compris, C-29-4-13), 29-4b quatre de code et sept comptée largement ;
  le découpage reste, la 29-4b porte une dérogation motivée (C-29-4-18).
- *Annotation du 2026-10-14 (validation P2, R1 = F1)* : `pool` sort du décompte avec l'AC11 ; la 29-4a touche **quatre**
  modules de code (les deux enveloppes neuves et la variante « archivé » vont dans des modules déjà comptés), sept
  distincts au total ; le découpage reste, la dérogation de la 29-4b reste (relue, inchangée), aucune n'est due pour la
  29-4a. Le signal D5 levé en P2 (HIGH né de la remédiation P1) est déclaré au Project Lead (Change Log).
- *Annotation du 2026-10-14 (validation P3, R4 = F4)* : le « quatre » de l'annotation précédente était faux **à sa
  date**, d'un module : la fixture `attendre_un_verrou_en_attente` que la même remédiation P2 ajoutait à
  `kesh-db/src/test_fixtures.rs` (compilé en permanence, hors `cfg(test)`) en faisait cinq. La P3 retire la fixture
  (C-29-4-21) : la 29-4a touche de nouveau **quatre** modules de code — et, comptée largement, six fichiers de
  production, deux doc-comments seuls s'ajoutant (R8 de la P3 ; C-29-4-18). Le découpage reste.

### C-29-4-7 — Résidu : un cycle à trois où l'import serait victime

- **Contexte** : import (partagé depuis son `INSERT`, en attente de l'intervalle d'une suppression d'import), suppression
  d'import (revérification en attente derrière la clôture), clôture en attente : cycle à trois sur un même compte.
- **Retenu** : assumé, écrit au doc-comment de la revérification ; la clôture, plus légère, est la victime désignée
  (prémisse tenue par le dépôt, `rejeu_interblocage_e2e.rs:9-11`), et elle est rejouée. Classe LOW : aucune issue.
- **Écartée** : rejouer l'import (extraction de ~160 lignes sur deux chemins, CAMT et CSV) — hors périmètre minimal.
- **Réversible** : oui (une enveloppe à ajouter si le cas se produit).
- *Amendé le 2026-10-14 (validation P1, F8 b)* : la réserve « et que la transaction la plus légère ne soit pas
  désignée » décrivait un cas qui n'arrive qu'à égalité de poids ; dans ce cycle, la clôture (une demande en attente,
  rien d'écrit) est toujours la plus légère. Le résidu est **théorique**, écrit comme tel, sans « à surveiller ».
- *Annoté le 2026-10-14 (validation P2, F2)* : le même cycle existe avec le `PATCH` du compte à la place de la clôture
  comme demandeur d'exclusif en file ; le `PATCH` est désormais rejoué (C-29-4-19), et l'import, qui a écrit, reste le
  plus lourd : même résidu théorique. L'argument de poids est écrit (fiche, § *Cycles*, F11 de la P2).
- *Annoté le 2026-10-14 (validation P3, F3)* : « la clôture est toujours la plus légère » est **raisonné** : le dépôt
  n'établit le choix du plus léger que dans des cycles à deux, et ce cycle-ci en compte trois. Si InnoDB ne comparait
  que la paire qui ferme le cycle, l'import ou la suppression pourraient être désignés. T29a-4 et T29a-9, premiers
  tests à trois, portent une seconde contingence (victime observée consignée, remontée à l'orchestrateur) ; le résidu
  reste écrit théorique sous cette réserve.

### C-29-4-8 — Pas de revérification là où un invariant ou un verrou existant tient

- **Retenu** : W30, W1-W6 et `accept_one_*` (I1), W20 (`pay_in_tx` sous `FOR UPDATE`, et I2), W8, W27, W13-W15
  (`FOR UPDATE` de la ligne) : aucun appel ; le troisième inventaire du registre les classe `SansFenetre` avec leur
  raison.
  *Annoté le 2026-10-14 (revue de code P1 de la 29-4a, A-5)* : la variante s'appelle `Fermee` au code
  (`bank_account_closed_inventory.rs`), renommée au développement (C-29-4-30).
- **Réversible** : oui.

### C-29-4-9 — Le critère décidable : trois volets

- **Retenu** : (1) verrou neuf déjà tenu ou dernière acquisition ; (2) tout cycle fermé a tous ses participants rejoués
  (ou passe par la clôture, rejouée) ; (3) aucune naissance de vue déplacée (hérité de l'AC7 de la 28-4). Il remplace,
  pour les chemins touchés, le volet « aucun verrou ajouté » de la 28-4, que fermer la fenêtre ne permet pas de tenir.
- **Réversible** : sans objet.

### C-29-4-10 — Égalité sur la clé primaire, contrôlée par `EXPLAIN`

- **Contexte** : un verrou d'intervalle (plan par l'index de société) ajouterait des verrous et romprait « déjà tenu ».
- **Retenu** : `WHERE id = ? AND company_id = ?` (accès `const` sur `PRIMARY`), et un test `EXPLAIN`.
- **Réversible** : oui.

### C-29-4-11 — Textes : manuel et API externe en 29-4b ; CHANGELOG par union dans chaque PR ; `[0.13.3]` intouché

- **Retenu** : la 29-4a écrit les doc-comments et la ligne du CHANGELOG de ce qu'elle ferme ; la 29-4b réécrit L1 au
  manuel (PDF), à `api-external.md` et aux doc-comments restants. La section `[0.13.3]` du CHANGELOG, la fiche et le
  registre de la 28-4 sont des relevés datés : ils ne se réécrivent pas.
- **Réversible** : oui.

### C-29-4-12 — Un seul test du cycle neuf (W25), non cinq

- **Contexte** : les cinq annulations ferment le même cycle (compte X → exercice contre exercice → compte S), avec des
  partenaires tous rejoués au sol.
- **Retenu** : T-b3 sur W25 prouve que le cycle est détecté (`1213`) et rejoué ; pour les quatre autres, la table des
  cycles et les enveloppes existantes (`audit_route_registry.rs`).
- **Écartée** : cinq tests à trois connexions (coût, sans information neuve).
- **Réversible** : oui.
- *Annotation du 2026-10-14* : le test se nomme désormais **T29b-3** (C-29-4-17).

### C-29-4-13 — Épingler `innodb_snapshot_isolation = OFF` par session dans `create_pool`

- **Contexte** (validation P1, R4 = F6 ; décision de l'orchestrateur : la revérification ne doit pas dépendre en silence
  de ce réglage) : sous `ON` — défaut de MariaDB à partir de la 11.6 —, une lecture verrouillante qui rencontre une
  version postérieure à la vue rend `ER_CHECKREAD` (1020), dans la course même que la revérification traite. Établi au
  code : `map_db_error` ignore 1020, qui sort en `DbError::Sqlx` → `500 INTERNAL_ERROR`. Constaté : base de dev
  `10.11.16`, variable présente, `OFF`. Le dépôt dépend déjà de `OFF` ailleurs (`routes/reconciliation.rs:2166`,
  `bank_imports.rs:439`). Version du NAS inconnue.
- **Retenu** : `configurer_session` (appelée par `after_connect`) exécute `SET SESSION innodb_snapshot_isolation = OFF`,
  en ignorant la seule erreur 1193 (variable inconnue d'un serveur ancien, qui se comporte comme `OFF`) ; test T29a-8
  (constat de 1020 sous `ON`, puis épinglage) ; mutation M29-5.
- **Écartées** : (1) traduire 1020 en `BANK_ACCOUNT_CLOSED` sur le chemin de la revérification — faux quand la
  modification postérieure est un `PATCH` ou un renommage, et ne couvre pas l'`INSERT`, dont le contrôle de clé
  étrangère est lui aussi une lecture verrouillante (non établi) ; (2) traduire 1020 en rejeu — l'import et la création
  de lot ne sont pas rejoués (C-29-4-7) ; (3) vérification au démarrage avec refus de démarrer — bloquerait une
  installation à la montée de version du serveur pour un réglage que l'application peut poser elle-même ;
  (4) exigence de version au manuel d'administration seule — laisse le `500` à qui ne lit pas le manuel.
- **Réversible** : oui (une instruction).
- *Annotation du 2026-10-14 (validation P2, R1 = F1, HIGH convergent ; décision de l'orchestrateur)* : **retiré de la
  story**. `create_pool` n'a **aucun appelant** — la production construit son pool dans `kesh-api/src/main.rs:74`, sans
  `after_connect`, depuis la Story 1.2 — : le réglage n'aurait atteint aucune connexion, et T29a-8 (b), qui appelait
  `configurer_session` directement, aurait certifié une prémisse fausse. Retrait plutôt qu'ajout : brancher le réglage
  sur le pool réel touche toute la base (`sql_mode`, `time_zone` décrits par `create_pool` et jamais appliqués) et
  ajoute un module à la 29-4a. Sortent : l'AC11, `configurer_session`, T29a-8, M29-5. Question portée par **#669**
  (bug, P3). Angle mort écrit : sous MariaDB 11.6+, la revérification rendrait `1020` → `500` dans la course, comme les
  deux sites préexistants. Les écartées (1) à (4) ci-dessus restent écartées pour la story ; #669 les rouvre.

### C-29-4-14 — La réouverture n'est pas rejouée

- Voir l'amendement de C-29-4-5 (validation P1, F5). Réversible : oui (une enveloppe).
- *Renversé le 2026-10-14 (validation P2, F3 = R9 ; décision de l'orchestrateur)* : **la réouverture est rejouée**.
  L'argument de la P1 était faux : « toute écriture sur un clos est refusée **avant** de prendre ce partagé » — W22 et
  W17 prennent le partagé du compte par leur `INSERT` avant que la revérification les refuse, les annulations le
  prennent par la revérification ; et une demande partagée attend derrière une demande exclusive **en attente**, celle
  de la réouverture comprise. Cycle à quatre (raisonné) : annulation → réouverture → import → route de gestion →
  annulation. Retenu : `reopen_once` et `retry_app_on_deadlock("bank_accounts::reopen", …)`, T29a-9, M29-20.
  **Écartée** : rejouer aussi les routes de gestion (`PUT`, `POST`, `DELETE /bank-accounts`) pour fermer le résidu de
  ce cycle — `M` y est à égalité de poids avec la réouverture et peut être désignée, `500` sans rien d'écrit ; quatre
  gestes concurrents sur un même compte et une même société, aucun cas observé : trois enveloppes de plus pour un cas
  négligeable, résidu écrit. Réversible : oui (une enveloppe).
- *Annoté le 2026-10-14 (validation P3, F2 ; décision de l'orchestrateur de garder l'écart)* : le résidu était
  inventorié pour la seule réouverture. Il se forme avec **tout** demandeur d'exclusif en file sur le compte —
  clôture, réouverture, `PATCH`, W8, W27 — et `M`, qui ne tient aucun verrou d'enregistrement, est à égalité de poids
  avec la clôture et la réouverture, **strictement plus légère**, donc victime désignée, contre le `PATCH`, W8 et W27.
  La décision ne change pas (routes de gestion non rejouées, quatre gestes concurrents, `500` sans rien d'écrit) ;
  l'inventaire du résidu est réécrit dans la fiche (critère volet 2, AC4, § *Cycles*, angles morts) et au doc-comment
  prescrit.
- *Annoté le 2026-10-14 (revue de code P1 de la 29-4a, E-1 ; décision de l'orchestrateur)* : **l'écart est levé** —
  les routes de gestion (`PUT`, `POST`, archivage) sont rejouées (C-29-4-28). Le résidu se ferme : `M`, désignée,
  rejoue. Ce qui a fait changer la décision n'est pas le cycle à quatre de la 29-4b mais le cycle **préexistant**
  `I ↔ M` (et `I → M → E → I`), atteignable dès la 29-4a, où `M` était la victime en `500`.

### C-29-4-15 — L'ensemble des gardes non verrouillantes s'extrait du code, et `FENETRE` lui est égal

- **Contexte** (validation P1, R3) : l'assertion (ii) de l'AC5 sélectionnait les sites par le **texte** des raisons du
  registre principal — que la 29-4b réécrit —, manquait W20 et rangeait W1-W6 / `accept_one_*`, qui sont des
  `Invariant`, à `FENETRE`.
- **Retenu** : `G` = appelants d'`ensure_open_in_tx` / `ensure_open_unarchived_in_tx` ∪ fonctions portant un littéral
  qui lit `closed_on` `FROM bank_accounts` sans `FOR UPDATE` ni `LOCK IN SHARE MODE` (hors `bank_accounts.rs`) ;
  assertion d'**égalité** `G` = fonctions de `FENETRE` ; neuf entrées au sol (W9, W17, W20, W22, W24, W25, W28, W29,
  W30).
- **Écartées** : un champ `Lecture::{Verrouillante, NonVerrouillante}` ajouté au registre principal (27 entrées à
  retoucher, et le champ resterait une déclaration, non une extraction) ; la sélection par texte (change de sens à la
  réécriture d'une raison).
- **Réversible** : oui.

### C-29-4-16 — La course archivage / import et lot (L5) est nommée, non fermée

- **Contexte** (validation P1, F8 a) : la revérification, partagée avec les annulations, ne lit pas `archived` ; un
  archivage commité entre la garde de tête de W22 / W17 et leur `INSERT` laisse passer l'écriture sur un compte archivé.
  Le dépôt nomme déjà cette course pour l'import (« L5 de la route d'archivage », `bank_accounts.rs:866-867`), sans issue.
- **Retenu** : angle mort écrit (fiche, Dev Notes ; doc-comment de `recheck_open_locked_in_tx`) ; **issue à ouvrir** par
  l'orchestrateur (course archivage / import et archivage / lot, L5).
- **Écartée** : une variante `recheck_open_unarchived_locked_in_tx` pour W22 / W17 — hors #651, et non mécanique (W17
  rend `IllegalStateTransition` pour un archivé, W22 `NotFound` : deux traductions).
- **Réversible** : oui.
- *Renversé le 2026-10-14 (validation P2, R2 = F4 ; décision de l'orchestrateur)* : la course **est fermée**, et la
  variante écartée est retenue — elle est mécanique aux conditions que la P2 a écrites (C-29-4-20). Plus d'issue L5 à
  ouvrir ; l'angle mort disparaît de la fiche ; le doc-comment `bank_accounts.rs:866-867` se réécrit en 29-4a.

### C-29-4-17 — Identifiants propres à la story : `T29a-n`, `T29b-n`, `M29-n`

- **Contexte** (validation P1, R7) : `T-a1` à `T-a5` existent dans le module `compte_clos` (28-4), `T-b1` à `T-b4` dans
  `bank-accounts.api.test.ts`, `M1` à `M15` dans les doc-comments de la 28-4.
- **Retenu** : renumérotation de la fiche ; `T-a31`, `T-a45`, `T-a48`, `T-a49` gardent leur sens (tests de la 28-4).
- **Réversible** : oui.

### C-29-4-18 — Dérogation à la règle de splitting pour la 29-4b

- **Contexte** (validation P1, R6 = F7) : comptée largement, la 29-4b touche sept modules.
- **Retenu** : pas de redécoupage ; ce qui dépasse cinq n'est que du texte (deux fichiers à doc-comments seuls, le
  manuel), le code est un rollout mécanique du patron de la 29-4a ; séparer textes et code ferait vivre plus longtemps
  des textes faux. Section *Dérogation règle de splitting (29-4b)* de la fiche.
- **Écartées** : une 29-4c pour les textes ; déplacer en 29-4a la réécriture de `bank_accounts.rs` (fausse tant que les
  annulations restent ouvertes).
- **Réversible** : oui.
- *Annoté le 2026-10-14 (validation P3, R8)* : la 29-4a, comptée largement, touche elle aussi plus de cinq fichiers de
  production — quatre modules de code et deux fichiers de doc-comments seuls (`kesh-db/src/errors.rs`,
  `kesh-api/src/routes/bank_imports.rs`), soit six. Même argument, même conclusion : ce qui dépasse cinq n'est qu'une
  phrase de doc-comment que la revérification rend incomplète ; la fiche étend la section *Dérogation* à la 29-4a.
- *Annoté le 2026-10-14 (validation P4, L4)* : le « sept » de la 29-4b comptée largement ne suivait aucune règle. Règle
  énoncée : fichiers de production (code ou doc-comments), sans documents ni tests — 29-4a **six**, 29-4b **six** (quatre
  de code, deux de doc-comments seuls), **neuf** avec ses trois documents. Conclusion inchangée.

### C-29-4-19 — Rejouer le `PATCH` du compte (décision de l'orchestrateur, validation P2)

- **Contexte** (validation P2, F2) : la revérification de la 29-4b crée un cycle à trois par la file d'attente — un import
  `I` qui a passé sa revérification tient le compte en partagé et attend l'intervalle d'`idx_bank_transactions_import`
  tenu par une suppression d'import `D` (W24) ; un `PATCH` du compte demande l'exclusif derrière `I` ; `D`, à sa
  revérification, attend derrière le `PATCH` en file. Le `PATCH`, qui n'a rien écrit, est le plus léger : victime,
  `500` sans enveloppe. C'est aussi lui la victime de l'angle mort « PATCH » par `accounts` écrit en P1.
- **Retenu** : `patch_once` (la transaction du `PATCH` : partagés des réglages, `FOR UPDATE` du compte, audit, commit)
  enveloppée par `retry_app_on_deadlock("bank_accounts::patch", …)` ; version et `validate_journal_account_id` devant
  l'enveloppe ; `rollback` explicite sur refus ; test T29a-10 (patron `victime()` : le `PATCH` tient les partagés des
  réglages que la transaction de test demande) ; mutation M29-21. Ferme le cycle et l'angle mort d'un seul geste.
- **Écartées** : écrire le cycle comme troisième angle mort (F2, option b : deux angles morts au lieu d'une enveloppe) ;
  monter le cycle à trois dans le test (le cycle à deux suffit à prouver que la victime est rejouée).
- **Réversible** : oui (une enveloppe).

### C-29-4-20 — Deux variantes de la revérification ; W17 convertit l'archivé (décision de l'orchestrateur, validation P2)

- **Contexte** (validation P2, R2 = F4) : la revérification de l'import et du lot doit lire `archived` (fermer L5), celle
  des annulations jamais (un archivé ne se rouvre pas : refuser l'annulation d'un de ses règlements la rendrait
  définitivement impossible).
- **Retenu** : `RECHECK_OPEN_LOCKED_SQL` / `recheck_open_locked_in_tx` (annulations) et
  `RECHECK_OPEN_UNARCHIVED_LOCKED_SQL` / `recheck_open_unarchived_locked_in_tx` (W22, W17), symétrie exacte
  d'`ENSURE_OPEN_SQL` / `ENSURE_OPEN_UNARCHIVED_SQL` ; ordre « archivé avant clos » ; W22 rend `NotFound` (traduit en
  `404` par la route, rien à ajouter) ; W17 convertit `NotFound` en `IllegalStateTransition("le compte bancaire source est
  archivé")` au site d'appel (sûr : la ligne existe après l'`INSERT` à clé étrangère, sa société est contrôlée à
  l'étape (0)) ; registre `Reverifiee(Variante, …)` et assertion (i bis) en dur pour les annulations.
- **Écartées** : une seule fonction qui rendrait `archived` à l'appelant (chaque appelant redécide, et une annulation
  pourrait le lire) ; faire lire `archived` à la fonction partagée (régression : annulations impossibles sur un
  archivé) ; traduire l'archivé de W17 en `NotFound` (changerait le refus de l'étape (0)).
- **Réversible** : oui.

### C-29-4-21 — Établir l'attente par `INNODB_TRX`, non par un délai (T29a-6, T29a-7)

- **Contexte** (validation P2, R4 = F7) : dans l'autre ordre, l'issue ne départage pas « prémisse vraie » de « montage
  arrivé avant que la demande exclusive soit en file » ; un délai de 500 ms l'espérait, et la sonde de requête en cours
  ne voit pas une requête bloquée (P5 de la 28-4).
- **Retenu** : la tâche de la clôture relève son `CONNECTION_ID()` ; le test attend une ligne d'`INNODB_TRX` avec
  `trx_mysql_thread_id` égal et `trx_state = 'LOCK WAIT'`, bornée à 10 s (fixture neuve `attendre_un_verrou_en_attente`).
  Colonnes présentes sous MariaDB 10.11 : raisonné, à vérifier au dev.
- **Écartées** : garder le délai seul (passe à vide sans signal) ; `INNODB_LOCK_WAITS` (même information, une jointure de
  plus) ; écrire seulement au Dev Agent Record que le test ne distingue pas les deux cas.
- **Réversible** : oui.
- *Annoté le 2026-10-14 (validation P3, R1 = F1 ; décision de l'orchestrateur)* : **la sonde `INNODB_TRX` est
  remplacée**. Elle exige le privilège `PROCESS`, que les droits versionnés de l'utilisateur de dev ne donnent pas
  (`scripts/mariadb-init/01-dev-grants.sql`, deux espaces de noms) — le dépôt le disait deux fois
  (`reconciliation_e2e.rs:7546-7553`, `:8394-8397`) ; « colonnes présentes : raisonné » visait le mauvais obstacle.
  **Retenu** : le helper existant `sonde_verrou_nowait` (`test_fixtures.rs:607`), en boucle locale au test, avec
  `SELECT id FROM bank_accounts WHERE id = ? LOCK IN SHARE MODE NOWAIT` sur une connexion tierce : compatible avec le
  seul partagé de l'écrivain, il échoue en `1205` dès que la demande exclusive de la clôture est en file. Aucun
  privilège, aucune fixture neuve — la fixture `attendre_un_verrou_en_attente` est retirée, et la 29-4a revient à quatre
  modules de code (C-29-4-6). **Établi** le 2026-10-14 sur la MariaDB de dev (`10.11.16`, base `kesh_294`, table
  jetable, trois connexions) : succès avec un partagé tenu, `1205` aussitôt avec un exclusif en file, succès après les
  commits ; à rejouer dans le test. **Repli**, si la branche rebasée le contredisait : le précédent
  `reconciliation_e2e.rs:7546-7553` (attente bornée constatée, issue après le commit) et le constat écrit au Dev Agent
  Record que le test ne distingue plus les deux cas. **Écartées** : accorder `PROCESS` à l'utilisateur de dev (droit
  global, contraire à la justification de `01-dev-grants.sql`, hors des modules de la story) ; garder `INNODB_TRX`
  parce que le conteneur de la station porte, ce jour, un `GRANT ALL PRIVILEGES ON *.*` à `kesh` étranger au fichier
  versionné (le test passerait ici et rougirait sur un conteneur rejoué).
- *Annoté le 2026-10-14 (validation P4, M1, L1, L5)* : la garde contre le montage à vide est une **sonde de référence
  entre les étapes (1) et (2)**, avant le lancement de la clôture — placée après, la clôture pouvait déjà être en file et
  la garde donner un faux rouge ; la forme `LOCK IN SHARE MODE NOWAIT` n'a aucun précédent dans le dépôt (constat hors dépôt
  seulement), elle s'établit au T0 du développement, repli inchangé ; le motif « 500 ms » du repli se lit à
  `reconciliation_e2e.rs:7538-7553`.

### C-29-4-22 — Un seul test « annulation sur un compte archivé » (W25), non cinq

- **Contexte** : les cinq annulations appellent la même fonction `recheck_open_locked_in_tx`.
- **Retenu** : T29b-5 sur W25 (règlement par virement, compte archivé ensuite — l'archivage ne compte que les
  `bank_transactions`) ; pour les quatre autres, l'assertion (i bis) du registre, qui interdit en dur la variante aux
  annulations ; mutation M29-24.
- **Écartée** : cinq tests fonctionnels (coût, sans information neuve).
- **Réversible** : oui.

---

*Choix du développement de la 29-4a (`bmad-dev-story`, Opus 5.5, autonomie, 2026-10-14).*

### C-29-4-23 — Répartition des tests de la partie a

- **Contexte** : l'AC3 place les tests de fenêtre au module `compte_clos` de `bank_accounts_repository.rs` ; ceux de
  W17 exigent une facture fournisseur payable et un compte source lié au grand livre, montage que porte déjà
  `payment_batches_repository.rs` (`setup`, `make_invoice`).
- **Retenu** : T29a-1, T29a-2, T29a-6, T29a-7, T29a-11 au module `compte_clos` ; T29a-3 et T29a-12 dans
  `payment_batches_repository.rs`, qui réutilisent son montage (le compte source y est rendu non principal par SQL,
  un principal ne se clôt pas). T29a-7 reste au module `compte_clos` : il insère son lot par SQL et n'a besoin
  d'aucune facture.
- **Écartée** : recopier le montage fournisseur dans `bank_accounts_repository.rs` (duplication).
- **Réversible** : oui (déplacement de tests).

### C-29-4-24 — La position d'une revérification, déclarée en structure et non en texte

- **Contexte** : l'AC5 écrit `Reverifiee(Variante, "<position>")` et exige un ordre textuel contrôlé (assertion
  (iii)).
- **Retenu** : `Reverifiee(Variante, Position { apres, avant })` — `apres` et `avant` sont des fragments SQL cherchés
  dans les littéraux de la fonction (aplatis : échappements neutralisés, blancs réduits, majuscules) ; l'appel doit
  suivre le dernier littéral qui contient `apres` et précéder le premier qui contient `avant`. La position devient
  l'entrée même de l'assertion, au lieu d'un texte que rien ne relirait. La 29-4b pourra élargir `Position` aux
  jetons de code et à la « dernière occurrence de tout geste verrouillant » de l'AC8.
- **Écartée** : une chaîne libre doublée d'une table d'ordre séparée (deux sources pour un fait).
- **Réversible** : oui.

### C-29-4-25 — Une conclusion commune aux trois tentatives ; `begin` et `commit` gardent leur rendu

- **Contexte** : `close_once`, `reopen_once` et `patch_once` concluent de la même façon (commit, ou `rollback`
  explicite avant de rendre l'erreur). Le patron `delete_once` rend les erreurs de `begin` / `commit` en
  `AppError::Database`, les trois routes en `AppError::Internal`.
- **Retenu** : une fonction `conclure(tx, result)` partagée (DRY) ; le rendu `Internal` de `begin` et `commit`
  **inchangé** — un `commit` InnoDB ne rend pas d'interblocage (les cycles se détectent à la demande de verrou),
  et changer le code d'erreur d'une panne de connexion est hors périmètre.
- **Écartée** : recopier le `match` final trois fois ; aligner sur `delete_once`.
- **Réversible** : oui.

### C-29-4-26 — T29a-4 et T29a-9 : attendre la demande de `t2` en cours avant que `t1` demande `R`

- **Contexte** : la séquence de l'AC4 enchaîne l'étape (3) (`t2` demande le compte en partagé, en tâche) et
  l'étape (4) (`t1` demande `R`) sans point de rendez-vous.
- **Retenu** : entre les deux, `attendre_une_requete_en_cours` sur la requête de `t2` — une aide d'ordonnancement, non
  une preuve (la sonde voit une requête en cours, non bloquée) ; c'est l'événement de rejeu de l'étape (5) qui
  établit l'attente, comme la fiche le prescrit. Le cycle se forme dans les deux ordres ; le rendez-vous évite
  seulement de dépendre de l'ordonnanceur pour que la demande de `t2` parte la première.
- **Écartée** : un délai fixe.
- **Réversible** : oui.

### C-29-4-27 — Dans les tests de fenêtre de l'import, la tâche commite sur succès

- **Contexte** : l'AC3 prescrit, pour W22, un `rollback` explicite de la tâche avant les comptages.
- **Retenu** : la tâche commite si l'import réussit — ce que fait la route — et annule explicitement sinon. Au sol,
  le test rougit sur l'issue (étape (5), `Ok(())` au lieu du refus) ; sous la story, l'annulation explicite précède
  les comptages, comme prescrit. Un `rollback` inconditionnel aurait rendu les comptages « rien d'écrit » vrais au
  sol aussi, sans rien prouver.
- **Écartée** : `rollback` inconditionnel.
- **Réversible** : oui.

### C-29-4-28 — Rejouer les routes de gestion du compte (décision de l'orchestrateur, revue de code P1, E-1)

- **Contexte** : la lentille E a reconstruit au code un cycle **préexistant** qu'aucun texte n'écrivait : un import ou
  un lot `I` tient la ligne du compte en partagé par le premier contrôle de clé étrangère de son `INSERT` et demande
  `companies` en partagé (le second) ; une route de gestion `M` (`PUT`, archivage, `POST` qui démote le principal)
  tient la sentinelle `companies` en exclusif et demande la ligne du compte en exclusif. À deux, `I ↔ M` ; à trois
  par la file, `I → M → E → I` quand la clôture, la réouverture ou le `PATCH` attend la ligne entre eux. `M`, non
  rejouée, plus légère que `I` (qui a écrit), en était la victime : `500`, rien d'écrit.
- **Retenu** : `create_once`, `update_once`, `archive_once` sous `retry_app_on_deadlock` (opérations
  `bank_accounts::create`, `::update`, `::archive`), sur le patron de la réouverture et du `PATCH`, conclusion par
  `conclure` ; contrôles de forme, onboarding, `validate_journal_account_id` et le pré-vol de l'archivage devant
  l'enveloppe. Tests T29a-13 (`PUT`), T29a-14 (archivage), T29a-15 (création), patron `victime()` et capture du rejeu ;
  mutations M29-26 à M29-28. Point (vi) du registre de rejeu : dix → treize. Résidu de C-29-4-14 fermé.
- **Écartée** : écrire le cycle comme résidu préexistant sans rejeu (remède de la lentille E).
- **Réversible** : oui (trois enveloppes).

### C-29-4-29 — La constante de lecture de `closing_checks.rs` entre au troisième inventaire (rebase sur la 29-1a)

- **Contexte** : au rebase sur `b4abae5d`, le troisième inventaire (T29a-5) a vu un site neuf : la constante
  `EXAMINED_ACCOUNTS_SQL` de `kesh-db/src/repositories/closing_checks.rs` (Story 29-1a) lit `closed_on` de
  `bank_accounts` sans verrou, hors de toute fonction. Ce n'est pas une garde : la préparation de la clôture d'un
  exercice liste les comptes à examiner, en lecture seule, et n'écrit rien.
- **Retenu** : une entrée `Fenetre::Fermee` pour `closing_checks.rs::<hors fonction>`, raison écrite ; décompte
  anti-muet de `G` : neuf → dix.
- **Écartée** : exclure le fichier de l'extracteur — une exclusion par fichier rendrait muet tout site futur du
  même fichier.
- **Réversible** : oui.

### C-29-4-30 — Écarts du développement à la fiche, versés au registre (revue de code P1, A-5)

- **Retenu, constaté au développement** : la variante `SansFenetre` de la fiche s'appelle `Fermee` au code (le nom
  dit le statut, non l'absence d'un site) ; trois mutations ajoutées à la campagne de la fiche — M29-3 bis (`LOCK IN
  SHARE MODE` retiré de `RECHECK_OPEN_UNARCHIVED_LOCKED_SQL`, la variante que W22 et W17 appellent : M29-3 ne visait
  que celle des annulations, sans appelant en 29-4a), M29-19 bis (`RECHECK_OPEN_LOCKED_SQL` en `FOR UPDATE`),
  M29-23 bis (W17 appelle `recheck_open_locked_in_tx`, la variante qui ne lit pas `archived`). Le Dev Agent Record les donnait « chacun au registre » sans qu'ils y soient.
- **Réversible** : oui.

### C-29-4-31 — Points reportés à la 29-4b par la passe ciblée P2 (annotation du 2026-10-14)

- **Retenu** : la passe ciblée P2 (Sonnet, 0/0/0/5) clôt la revue de la 29-4a ; R-3 est corrigé (CHANGELOG). Sont
  reportés à la 29-4b, sans toucher au code maintenant (fiche, T6-bis) : R-1 (T29a-15 joue un cycle synthétique, à dire
  dans le test et au point (vi)), R-2 (`was_primary` lu au pré-vol et réutilisé à chaque tentative d'`archive_once` : à
  relire dans la transaction de chaque tentative, avec un test — défaut préexistant que le rejeu élargit), R-4
  (doc-comments de cycles : distinguer mesuré et supposé), R-5 (mutation du `rollback` de `conclure`), et la liste des
  routes rejouées de `docs/api-external.md`.
- **Écartée** : corriger R-2 dans la 29-4a — chaque correctif de production rouvrirait la boucle de revue.
- **Réversible** : oui.


*Choix du développement de la 29-4b (Opus 5.5, autonomie, 2026-10-14).*

### C-29-4-32 — Les tests de fenêtre des annulations : dans `bank_accounts_closed_e2e.rs`, l'annulation appelée au dépôt

- **Contexte** : AC7 prescrit les tests T29b-1 « au dépôt, patron de l'AC3 », répartition au choix du dev. Les états à
  annuler (rapprochement `Entry`, règlement par virement, facture fournisseur payée par virement) se construisent par
  plusieurs routes ; `bank_accounts_closed_e2e.rs` en porte déjà les montages (`Ctx::rapprocher`, `regler`,
  `payer_fournisseur`).
- **Retenu** : T29b-1 (cinq tests), T29b-2 (deux) et T29b-5 dans `crates/kesh-api/tests/bank_accounts_closed_e2e.rs`.
  L'état se pose par les routes, sur un compte ouvert ; l'**annulation s'appelle au dépôt** (la fonction `_in_tx` que la
  route appelle, énumération `Annulation`), dans une transaction que le test possède — il voit le `DbError` et annule
  explicitement avant tout comptage ; la clôture concurrente est `close_in_tx`, au dépôt. T29b-5 passe, lui, par la
  route (l'annulation sur un compte archivé est un comportement de route). T29b-3 dans `rejeu_interblocage_e2e.rs`.
- **Écartée** : reconstruire ces états au dépôt dans `kesh-db/tests` (trois fichiers, montages de factures, d'écritures
  et de règlements à dupliquer) ; passer l'annulation par la route dans T29b-1 (l'erreur de dépôt, et le rollback
  explicite que l'AC3 exige, ne se voient plus).
- **Réversible** : oui (déplacement de tests).

### C-29-4-33 — R-2 : `is_primary` relu sous un `FOR UPDATE` avancé, par une fonction de dépôt neuve

- **Contexte** : T6-bis, R-2 de la passe ciblée P2 de la 29-4a — la garde « principal + autres comptes actifs » de
  l'archivage décidait sur le `is_primary` du pré-vol, réutilisé à chaque tentative du rejeu.
- **Retenu** : `bank_accounts::primary_flag_locked_in_tx` (`SELECT is_primary … AND archived = FALSE FOR UPDATE`),
  appelée dans `archive_once` juste avant la garde ; `None` → `404 BANK_ACCOUNT_NOT_FOUND`, le refus même
  qu'`archive_for_company` aurait rendu sur la même ligne. L'exclusif de la ligne, qu'`archive_for_company` prend
  ensuite, est **avancé** devant la garde : l'ordre « `companies` X puis compte X » est inchangé, et entre les deux
  positions la tentative ne fait qu'une lecture non verrouillante (raisonné). L'ordre des refus est inchangé
  (412 transactions → 412 principal → 409 version). Test `archive_rereads_the_primary_flag_inside_its_transaction`
  (`bank_accounts_e2e.rs`), rouge au sol : `412` au lieu de `409`.
- **Écartées** : une lecture non verrouillante après la sentinelle (juste tant que toute promotion passe par la
  sentinelle — l'onboarding `upsert_primary_in_tx` ne la prend pas) ; décider sur le `before.is_primary`
  d'`archive_for_company` (la garde passerait après le contrôle de version : ordre des refus changé).
- **Constat** : l'effet au sol est un **mauvais code de refus** (`412` sur un compte qui n'est plus principal), non un
  archivage indu — le verrou optimiste d'`archive_for_company` refusait déjà un compte dont la version avait bougé.
- **Réversible** : oui.
- *(Annotation du 2026-10-14, revue de code P1 de la 29-4b, A-5 et B-2.)* **Écart à la prescription** : T6-bis
  demandait « une promotion concurrente du compte **entre deux tentatives** » ; le cas joué est une **démotion**
  commitée entre le **pré-vol** et la **première** tentative (l'archivage attend la sentinelle). Le correctif est le
  même (relecture dans chaque tentative) ; le cas « entre deux tentatives » n'est pas joué. Et le test prouve la
  relecture **dans la transaction**, non le `FOR UPDATE` : un `SELECT` simple au même endroit y rendrait le même `409`
  (les bascules des routes de gestion sont déjà sérialisées par la sentinelle) ; le `FOR UPDATE` couvre l'intégration
  (`upsert_primary_in_tx`, sans sentinelle), non jouée. Écrit au doc-comment de `primary_flag_locked_in_tx`.
- *(Annotation du 2026-10-14, passe ciblée P2 de la revue de la 29-4b, R-1.)* **Prémisse fausse, retirée** : l'onboarding
  prend la sentinelle — `lock_company` (`FOR UPDATE` sur `companies`, `routes/onboarding.rs:585`) avant
  `upsert_primary_in_tx` (`:586`). Aucune promotion de production n'échappe à la sentinelle ; le motif « écarté » de la
  lecture non verrouillante (l. 482-483) et la « couverture de l'intégration » ci-dessus ne tiennent donc pas. Le
  `FOR UPDATE` reste, sans effet de justesse propre (il avance le verrou de la ligne qu'`archive_for_company` prend de
  toute façon) ; le doc-comment de `primary_flag_locked_in_tx` est corrigé.

### C-29-4-34 — R-5 : le `rollback` explicite de `conclure` gardé par un test textuel, son effet dynamique déclaré non couvert

- **Contexte** : R-5 demandait une mutation de `conclure` sans `rollback`, tuée par un test qui sonde le verrou, ou
  déclarée non couverte.
- **Établi au code** (sqlx 0.8.6) : `Transaction::drop` ne fait que mettre le `ROLLBACK` en file
  (`sqlx-mysql`, `start_rollback`), et `PoolConnection::drop` rend la connexion par une tâche lancée à part
  (`pool/connection.rs:208`) qui l'envoie (`ping`, `:314`). Sans `rollback` explicite, le verrou est rendu **un peu plus
  tard** ; un test qui le sonderait après la réponse ferait la course contre cette tâche.
- **Retenu** : T29b-7 (`bank_account_closed_inventory.rs`), test textuel sur les corps aplatis — `conclure` annule sur
  erreur, et les six tentatives s'y concluent ; il tue M29-35. Le comportement « verrou rendu avant la réponse » est
  déclaré non couvert dynamiquement (doc-comment du test, Dev Agent Record).
- **Écartée** : une sonde après la réponse (verte ou rouge selon l'ordonnanceur).
- **Réversible** : oui.

### C-29-4-35 — La position « en dernier » des annulations : une variante de `Position`, et deux jetons de plus

- **Contexte** : AC8 étend l'ordre textuel aux annulations (dernier geste verrouillant, avant l'audit) ; le registre de
  la 29-4a ne connaissait que `Position { apres, avant }` (littéraux).
- **Retenu** : `Position` devient une énumération — `Litteraux { apres, avant }` (import, lot) et `EnDernier` (les cinq
  annulations), contrôlée par `fautes_en_dernier` : jetons SQL de la fiche dans les littéraux, appels dans la copie
  masquée ; un corps sans aucun geste verrouillant est une faute (anti-muet). Aux appels de la fiche
  (`reverse_in_tx(`, `reverse_owned_in_tx(`, `cancel_settlement_in_tx(`) s'ajoutent les deux dissolutions du lettrage de
  pièce (`dissolve_invoice_document_group_in_tx(`, `dissolve_supplier_invoice_document_group_in_tx(`), qui écrivent :
  sans effet sur l'issue (elles précèdent la contre-passation), un angle mort de moins. Volet inverse :
  `the_last_acquisition_rule_reports_each_fault_on_built_cases` (sept cas).
- **Écartée** : une variante `Fenetre` de plus (la fiche écrit `Reverifiee(Variante::Clos, …)`).
- **Réversible** : oui.

### C-29-4-36 — Le paragraphe « Deux limites » du manuel devient « Deux précisions »

- **Contexte** : AC9 remplace la seconde puce (la fenêtre de #651) ; l'intitulé « Deux limites » ne disait plus vrai
  de cette puce.
- **Retenu** : « Deux précisions. » ; la première puce (boutons affichés, enregistrement refusé) est inchangée.
- **Réversible** : oui.

### C-29-4-37 — La moitié W25 de T29b-2 tue aussi M29-13, par son montage

- **Contexte** : la fiche (R5 de la P2) écrit que la moitié W25 ne tue pas M29-13, l'issue de la clôture (`Done`) étant
  la même sous la mutation.
- **Retenu** : le patron commun de T29b-2 **exige** que la clôture soit vue en attente sur son `FOR UPDATE` (étape de
  l'AC7) ; sous M29-13, elle n'attend pas, et le montage échoue — pour les deux moitiés. Dit au doc-comment de la moitié
  W25 : c'est l'attente, non l'issue, qui distingue.
- **Réversible** : sans objet (constat).

### C-29-4-38 — Le volet inverse « en dernier », jeton par jeton : une table de cas écrite à part des listes

- **Contexte** : revue de code P1 de la 29-4b, B-1 (MEDIUM) — le volet inverse de T29b-4 n'éprouvait que quatre des dix
  jetons de `GESTES_VERROUILLANTS_SQL` et `GESTES_VERROUILLANTS_CODE` ; retirer l'un des six autres laissait tout vert.
- **Retenu** : `the_last_acquisition_rule_sees_each_locking_token` — une table de cas (cinq littéraux, cinq appels)
  **écrite dans le test, indépendamment des listes**, un geste du seul jeton après la revérification, faute exacte
  « précède « <jeton> » » exigée ; décompte anti-muet (dix cas) ; puis contrôle de couverture dans les deux sens (chaque
  entrée des listes a son cas). Retirer un jeton d'une liste rougit **par son cas** (« aucun geste verrouillant ») ;
  ajouter un jeton sans cas rougit par la couverture. Prouvé : M-B1a (`INSERT INTO`) et M-B1b (`reverse_in_tx(`)
  tuées (`294b-mutations.log`).
- **Écartée** : boucler sur les listes elles-mêmes pour construire les cas — retirer un jeton retirerait son cas, et le
  test resterait vert.
- **Réversible** : oui.

### C-29-4-39 — T29b-1 exige que l'annulation soit vue en attente

- **Contexte** : revue de code P1 de la 29-4b, E-4 et A-9 — `annuler_pendant_une_cloture` ignorait le résultat
  d'`attendre_une_requete_en_cours`, qui rend `false` quand la tâche a fini sans attendre (et panique à dix secondes).
- **Retenu** : `false` fait rougir (la clôture est d'abord annulée, puis le test échoue en citant l'issue de
  l'annulation). Au code corrigé, l'annulation **doit** attendre : la clôture tient l'exclusif de la ligne que la
  revérification demande. Le doc-comment du patron ne dit plus « ou terminée ».
- **Écartée** : `attend || tache.is_finished()` (l'issue « finie sans attendre » est précisément celle où la garde de
  tête, non la revérification, aurait pu refuser).
- **Réversible** : oui.


---

## Story 29-6 — La réconciliation ne propose plus une facture qu'un trop-perçu de quelques centimes fait refuser ([#665])

### C-29-6-1 — Prémisses exactes ; une précision et un doc-comment faux

- **Contexte** : #665 (2026-10-14) décrit la tolérance de ± 5 centimes à la proposition et le refus du trop-perçu à
  l'acceptation.
- **Retenu** : établi au code sur `fcd2ae75`, les quatre prémisses sont exactes. Précision écrite dans la fiche : la
  tolérance est symétrique, et seul son côté « le paiement dépasse le reste » est refusé ; l'autre côté donne un
  règlement partiel accepté. Le doc-comment de `find_unpaid_invoices_for_window` (« la tolérance couvre l'écart à son
  arrondi au centime, au plus 0.005 ») est faux depuis l'origine (0,05 contre 0,005) : réécrit par l'AC2.
- **Réversible** : sans objet (constat).
- *Annotation du 2026-10-14, validation P1 (R6)* : « faux depuis l'origine » était inexact — la phrase date de
  `eecf40cb` (Story 25-4-c), la tolérance de 0,05 de `22fd6d4a` (Story 8-4) : faux **depuis son écriture**.

### C-29-6-2 — Voie 1 : la proposition écarte le trop-perçu ; voie 2 écartée

- **Contexte** : #665 offre deux voies — la proposition écarte les trop-perçus, ou l'acceptation accepte l'écart toléré
  sur le compte d'arrondi (comme `SettlesWithRounding`). Consigne : établir au code les conséquences comptables, choisir
  le périmètre minimal sûr ; une écriture de produit d'arrondi sans compte d'arrondi configuré est exclue.
- **Retenu** : voie 1. Rien de nouveau ne s'écrit ; la proposition ne charge que ce que `classify_payment` admet.
- **Écartée** : voie 2, pour quatre raisons établies au code : (a) sans compte d'arrondi, `rounding_account_for_write`
  refuse encore (`ROUNDING_ACCOUNT_NOT_CONFIGURED`), donc le défaut subsiste sauf à faire lire les réglages par la
  proposition ; (b) le compte d'arrondi reçoit aujourd'hui des écarts d'un demi-centime au plus — un trop-perçu de
  5 centimes est une dette envers le client, pas une différence d'arrondi, et le passer en produit est une décision
  comptable qui revient à Guy ; (c) `classify_payment` est partagée avec le règlement manuel : l'élargir change la
  fiche de la facture, ne pas l'élargir refait deux règles du trop-perçu (#476 les avait réunies) ; (d) elle contredit
  « le trop-perçu est refusé, jamais écrit » (code, `api-external.md`) et « pas de tolérance de montant » (manuel,
  lettrage).
- **Réversible** : oui — la voie 2 peut venir plus tard comme évolution, sur demande.

### C-29-6-3 — La borne basse : une fonction Rust réciproque de `classify_payment`, liée au SQL, avant le `LIMIT 50`

- **Contexte** : `classify_payment(P, R)` rend `Overpayment` ⇔ `R < P − 0,005` (pour `P > 0` au centime, arrondi
  `MidpointAwayFromZero`). Le dépôt interdit un `ROUND` en SQL (une seule définition de l'arrondi).
- **Retenu** : `invoice_settlements::least_due_without_overpayment(paid) = paid − HALF_CENTIME`, à côté de
  `classify_payment`, liée en paramètre du `HAVING` ; un test unitaire exhaustif (grille au dix-millième) tient
  l'équivalence entre les deux lecteurs.
- **Écartées** : (a) filtrer en Rust après la requête par `classify_payment` — des trop-perçus occuperaient des places
  du `LIMIT 50` et feraient disparaître une facture valable au-delà ; (b) `ROUND(amount_due, 2) = ?` en SQL — seconde
  définition de l'arrondi ; (c) une borne basse égale au montant exact — écarterait le reste à quatre décimales soldé
  au centime (10.01 sur 10.0050, `SettlesWithRounding`), que l'acceptation admet.
- **Réversible** : oui (une fonction, un `HAVING`).

### C-29-6-4 — Le côté du paiement partiel ne change pas

- **Contexte** : un reste supérieur au paiement de 1 à 5 centimes est accepté en règlement partiel (`Ordinary`) ; la
  facture reste ouverte pour la différence.
- **Retenu** : inchangé — ce n'est ni un refus ni une écriture fausse, donc pas le défaut de #665. La tolérance de
  5 centimes ne borne plus que ce côté (constante `AMOUNT_TOLERANCE_HUNDREDTHS`, nom conservé, doc réécrit).
- **Écartées** : retirer la tolérance (les clients qui paient le montant QR arrondi d'une facture partiellement réglée
  ne seraient plus reconnus ; hors périmètre) ; l'élargir.
- **Réversible** : oui.

### C-29-6-5 — Signature : paramètre renommé, position conservée

- **Contexte** : `find_unpaid_invoices_for_window(…, tx_amount, amount_tolerance)` a treize sites d'appel : un en production (`routes/reconciliation.rs:656`), douze en test (`reconciliation_repository.rs` dix, `invoice_write_off.rs` deux).
- **Retenu** : renommer le paramètre `underpayment_tolerance` (le nom dit ce qu'il borne désormais), sans changer sa
  position ni son type : aucun site d'appel ne change.
- **Écartée** : une structure `InvoiceAmountWindow` sur le patron d'`InvoiceDateWindow` — plus de surface pour la même
  règle, et plus de conflits de rebase avec la 29-2, qui modifie la même requête et `InvoiceDateWindow`.
- **Réversible** : oui.

### C-29-6-6 — Aucun texte d'écran, aucune clé i18n, aucune capture

- **Contexte** : le libellé de `RECONCILIATION_OVERPAYMENT` (« Le montant de la transaction dépasse le reste dû de la
  facture. ») reste juste pour un appel direct ou une course.
- **Retenu** : le frontend ne change pas ; la frontière HTTP est exercée par le test `spawn_app` d'AC6 ; l'E2E complet
  tourne au dernier commit de code.
- **Réversible** : sans objet.
- *Annotation du 2026-10-14, validation P1 (R1)* : **remplacé en partie par C-29-6-10** — l'écran change (une case,
  une clé i18n, un spec E2E et ses captures). Le libellé de `RECONCILIATION_OVERPAYMENT` reste, lui, inchangé.

### C-29-6-7 — Le contournement écrit au manuel : régler le reste dû, puis rejeter la transaction

- **Contexte** : un client qui a payé quelques centimes de trop n'est plus proposé ; le règlement manuel refuse aussi
  le trop-perçu.
- **Retenu** : le manuel dit d'enregistrer le règlement du reste dû depuis la fiche, de **rejeter** la transaction à la
  réconciliation, et de traiter l'écart (remboursement ou écriture manuelle) — même forme que le contournement de la
  29-2 (C-29-2-8).
- **Réversible** : oui (texte).
- *Annotation du 2026-10-14, validation P1 (R1)* : le geste était **impraticable** à l'écran (aucune case sur une ligne
  sans proposition) ; il le devient par C-29-6-10. Le texte du manuel nomme désormais la case de la ligne «~Aucune
  correspondance~».

### C-29-6-8 — Les angles morts #665 écrits par la 29-3 se retirent au rebase

- **Contexte** : la fiche 29-3 (C-29-3-7) prescrit d'écrire au manuel (`:2049`) et au doc-comment de `candidate_rank`
  qu'une facture que le paiement dépasse est proposée puis refusée (#665). La 29-6 se développe après elle.
- **Retenu** : la 29-6 retire ou réécrit ces phrases au rebase (AC7, § *Zones partagées*), inventaire par
  `grep -n 665`.
- **Écartée** : demander à la 29-3 de ne pas les écrire — elle se merge avant, et ses phrases sont vraies à sa date.
- **Réversible** : sans objet.

### C-29-6-9 — Story ajoutée à l'Epic 29, sortie dans la v0.13.4

- **Contexte** : décision de l'orchestrateur du 2026-10-14 ; #665 relevée pendant la fiche de la 29-3.
- **Retenu** : clé `29-6-trop-percu-centimes` sous `epic-29` ; paragraphe « 29-6 » au fichier d'epic (§ *Les stories*,
  § *Ordre*, § *Releases*) ; ordre de développement 29-2 → 29-3 → 29-5 → 29-6.
- **Réversible** : oui.

### C-29-6-10 — Le périmètre s'étend à l'écran : une ligne sans proposition se coche, pour *Rejeter* seulement

- **Contexte** : validation P1, MEDIUM R1, **vérifié au code par l'orchestrateur** — une ligne « Aucune correspondance »
  n'a pas de case (`ReconciliationProposals.svelte:318-330`) et `onReject` n'agit que sur les lignes cochées : le geste
  « régler depuis la fiche, puis rejeter la transaction », prescrit par la 29-6 (AC7) et par la 29-2 (C-29-2-8), est
  impraticable. **Décision de l'orchestrateur** (consignée comme telle) : la 29-6 étend son périmètre pour rendre ce
  rejet possible ; *Accepter* reste sans effet ou désactivé pour une telle ligne.
- **Retenu** : AC10. Établi au code d'abord : `POST /reconciliation/reject` ne lit aucune candidate (pré-vol
  `classify_batch_transaction`, `UPDATE` de `reject_batch`), ses refus vont dans `failed[]` en `200` ; le test
  `post_reject_marks_transactions_as_manually_reviewed` rejette déjà des transactions sans facture ; *Annuler le rejet*
  (28-5b) ne lit pas non plus de candidate. Donc **aucun changement d'API** — l'écran seul change.
- **Écartées** : (a) écrire au manuel que la transaction reste en attente, sans geste (correction (a) de R1) — laisse
  une ligne qu'aucun geste n'écarte, et la 29-2 avec un contournement faux ; (b) ouvrir une issue `enhancement` et ne
  prescrire le geste qu'après (correction (b) de R1) — reporte hors de la v0.13.4 un geste que deux stories de la même
  release prescrivent ; (c) un bouton *Rejeter* propre à chaque ligne sans proposition — un second chemin vers la même
  route, un libellé et un test de plus pour le même effet.
- **Réversible** : oui (un composant, une clé).
- **Annotation du 2026-10-14 (validation P2, R1/F2)** : « pour *Rejeter* seulement », au titre, est inexact — la ligne
  porte aussi *Affecter manuellement* et *Éclater*, et les lignes sans proposition comprennent les débits et le non-CHF
  sans règle (`routes/reconciliation.rs:640-652`). Lire : une ligne sans proposition se coche ; *Accepter* ne la traite
  pas. Le rejet laisse la transaction en attente (#670, C-29-6-17).

### C-29-6-11 — *Accepter* grisé tant qu'une ligne sans proposition est cochée, et la page le dit

- **Contexte** : une fois la case ouverte à toute ligne, *Accepter* doit ne rien faire de faux pour une ligne sans
  proposition. Au sol, `onAccept` lève « No candidate » (`:205-207`), affiché en erreur.
- **Retenu** : `disabled` dès que la sélection contient une ligne sans proposition, garde défensive en tête
  d'`onAccept`, et une indication sous les boutons (« ne peut qu'être rejetée : décochez-la pour accepter les autres »),
  clé `reconciliation-labels-reject-only-hint` dans les quatre catalogues. Le compteur reste `selected.size`.
- **Écartées** : (a) *Accepter* n'envoie que les lignes avec proposition et ignore les autres — le plus simple au code,
  mais **silencieux** : la ligne cochée ne bouge pas et rien ne dit pourquoi, et le compteur « Accepter (n) » mentirait ;
  (b) grisé sans indication — un bouton inerte sans raison visible ; (c) une case grisée pour *Accepter* seulement —
  une case n'a pas deux états de sélection.
- **Réversible** : oui.
- **Annotation du 2026-10-14 (validation P2, R1/F2, F1, F4)** : le texte retenu ci-dessus (« ne peut qu'être
  rejetée ») et la clé `reconciliation-labels-reject-only-hint` sont **remplacés** : « ne peut pas être acceptée »,
  clé `reconciliation-labels-unmatched-not-acceptable-hint`, `data-testid` `reconciliation-unmatched-not-acceptable-hint`
  (C-29-6-16) ; l'indication passe « sous les boutons » → dans la rangée des boutons (C-29-6-18). La garde défensive
  n'est pas inatteignable en Vitest (`fireEvent.click` atteint un bouton désactivé) : éprouvée par M12.

### C-29-6-12 — Cinq modules : la limite, pas de découpage

- **Contexte** : la règle de splitting préventif découpe une story qui touche **plus de cinq** modules. Avec l'AC10 :
  `kesh-db`, `kesh-api`, `frontend/…/reconciliation`, `kesh-i18n`, documentation.
- **Retenu** : cinq, non dépassé ; pas de découpage. La 29-6a (écran) / 29-6b (proposition) a été pesée : les deux
  moitiés sont indépendantes au code, mais le manuel et le CHANGELOG de la proposition prescrivent le geste de l'écran —
  aucune ne se livre vraie seule, et l'AC10 tient en un composant sans API.
- **Écartée** : découper — deux PR, deux boucles de revue, et un ordre de merge imposé pour un texte qui ne vaut
  qu'ensemble.
- **Réversible** : oui — si une passe de revue étend encore le périmètre, la story découpe ou écrit une dérogation.
- **Annotation du 2026-10-14 (validation P2, R8/F8)** : granularité déclarée — crate Rust et paquet npm (`kesh-db`,
  `kesh-api`, `frontend`, `kesh-i18n`, documentation à part) : cinq. Au module métier, six ou sept ; si cette
  granularité est tenue pour la bonne, la fiche écrit une dérogation sur l'argument ci-dessus.

### C-29-6-13 — L'invariant « filtre avant `LIMIT 50` » est gardé par un test, pas déclaré

- **Contexte** : validation P1, R4 — aucun test ne tuait un filtre déplacé en Rust après la requête.
- **Retenu** : un second test de dépôt (50 factures trop-perçues puis une valide, la dernière en `(date, id)`) et la
  mutation M8.
- **Écartée** : déclarer l'invariant non gardé — c'est le motif même qui a écarté l'alternative Rust (C-29-6-3) ;
  laisser sans garde la raison d'un choix, c'est laisser le choix se défaire en silence.
- **Réversible** : oui.

### C-29-6-14 — La v0.13.4 réunit 29-2, 29-3, 29-4, 29-5 et 29-6

- **Contexte** : validation P1, R7 — le fichier d'epic (§ *Releases*) omettait la 29-5, la clé `epic-29` du registre
  disait « 29-2..4 ».
- **Retenu** : composition donnée par l'orchestrateur, écrite au fichier d'epic et au commentaire de `epic-29`.
- **Réversible** : oui.

### C-29-6-15 — Un spec E2E neuf, sur le patron de `reconciliation-unreject.spec.ts`

- **Contexte** : l'AC10 change ce qu'un utilisateur peut cocher et envoyer ; seul un E2E voit l'identifiant d'une ligne
  sans proposition traverser la frontière HTTP.
- **Retenu** : `reconciliation-reject-unmatched.spec.ts`, relevé unique, rejet puis *Annuler le rejet*, captures clair et
  sombre hors dépôt.
- **Écartée** : ajouter un scénario au spec de la 28-5b — il rejette par l'API à dessein (il éprouve le détail de
  l'import, pas la liste), et le mêler brouillerait ce qu'il garde.
- **Réversible** : oui.

### C-29-6-16 — L'indication dit « ne peut pas être acceptée », et ne nomme aucun autre geste

- **Contexte** : validation P2, R1 et F2 (convergents, MEDIUM) — « ne peut qu'être rejetée » est faux à l'écran : la
  ligne porte *Affecter manuellement* et *Éclater* ; pour un débit sans règle, le texte poussait vers un rejet sans
  écriture au lieu de l'imputation.
- **Retenu** : dire la seule vérité de la sélection — fr-CH « Une transaction sans correspondance ne peut pas être
  acceptée : décochez-la pour accepter les autres. » (de, it, en au même sens) ; clé
  `reconciliation-labels-unmatched-not-acceptable-hint`, `data-testid` `reconciliation-unmatched-not-acceptable-hint`.
- **Écartées** : (a) la version longue de R1 (« …ou imputez-la par *Affecter manuellement* ou *Éclater* ») — pour un
  paiement de client, la FAQ du manuel (`:2910`) interdit précisément ces deux gestes : l'indication pousserait le cas
  même de #665 vers une imputation fausse ; les boutons sont d'ailleurs visibles sur la ligne ; (b) garder la clé
  `…-reject-only-hint` avec un texte neuf — le nom reproduirait l'erreur.
- **Réversible** : oui (une clé, quatre valeurs).

### C-29-6-17 — La transaction rejetée reste en attente : angle mort assumé, renvoyé à #670

- **Contexte** : validation P2, R2 (MEDIUM) — une transaction rejetée reste `pending`, empêche la clôture du compte
  bancaire (`bank_accounts.rs:1025`, manuel `:1690-1694`), et aucun geste ne la rapproche d'un règlement saisi depuis la
  fiche ; le texte prescrit par la P1 (« — sans quoi la transaction reste en attente ») promettait l'inverse.
  L'orchestrateur a ouvert [#670] (bug P2) ; son intégration à l'epic est une décision attendue de l'auteur.
- **Retenu** : la 29-6 **ne ferme pas** #670 et n'en dépend pas ; elle l'écrit comme angle mort assumé (statut, AC10,
  inventaire (b), exclusions) et réécrit les textes prescrits (manuel `:2029`, `:2057`, `:2910`, CHANGELOG) pour ne
  plus dire que le rejet règle la transaction : il la retire de la liste et empêche qu'elle soit passée une seconde
  fois ; rejetée, elle reste en attente et empêche la clôture du compte bancaire. Au manuel, **sans numéro d'issue**
  (renvoi `\S\ref{sec:clore-compte-bancaire}`) ; au CHANGELOG, avec le lien #670.
- **Écartées** : (a) intégrer #670 à la 29-6 — ni mécanique (geste neuf de rattachement, ou statut neuf compté par la
  clôture), ni décidé, et la règle de découpage est à sa limite (C-29-6-12) ; (b) attendre la décision sur #670 — la
  fiche resterait bloquée sur un arbitrage qui ne change pas son code ; (c) un numéro d'issue au manuel — il périme à la
  fermeture de #670 sans que rien ne le signale.
- **Réversible** : oui — si #670 est intégrée avant le tag, les phrases d'attente se réécrivent avec elle.
- *Annotation du 2026-10-14 (passe ciblée P3, P3-3)* : la décision n'est plus attendue — Guy a intégré #670 à l'Epic 29
  comme **Story 29-8**, qui réécrira « puis rejetez la transaction » en « puis rattachez ». La 29-6 garde ses textes ;
  la 29-8 les remplace.

### C-29-6-18 — L'indication se place dans la rangée des boutons

- **Contexte** : validation P2, F4 — un paragraphe qui apparaît au cochage entre les boutons et la table descend la
  table d'une ligne sous le pointeur ; le clic suivant peut cocher la ligne voisine. R9/F4 : l'ordre avec l'indication
  de la 29-3 n'était pas fixé.
- **Retenu** : dernier enfant du `div` des boutons (`flex-wrap items-center`) ; ordre fixé : rangée des boutons,
  paragraphe de priorité de la 29-3, table. À largeur étroite, un retour à la ligne peut encore décaler la table :
  accepté, vérifié sur les captures.
- **Écartées** : (a) réserver la place en permanence (élément vide rendu sans condition) — la mutation M11 et son test
  (« indication absente ») perdraient leur objet, et une hauteur fixe ne tient pas les quatre longueurs de texte ;
  (b) l'indication sous la table — loin du bouton grisé qu'elle explique.
- **Réversible** : oui.

### C-29-6-19 — Le spec `reconciliation-deleted-import` garde sa règle ; seul son commentaire change

- **Contexte** : validation P2, R4 — son commentaire (`:12-14`) dit que la règle d'affectation rend les transactions
  sélectionnables ; après l'AC10, toute ligne l'est.
- **Retenu** : réécrire le commentaire (la règle n'est plus nécessaire à la sélection ; le ciblage par `data-tx-id`
  isole les lignes du spec) ; garder la règle et le code du spec.
- **Écartée** : retirer la règle — modifie un spec qui éprouve autre chose (un lot dont une transaction a disparu),
  hors du périmètre minimal, pour un gain nul.
- **Réversible** : oui.

### C-29-6-20 — La campagne de mutations tourne en gates ciblés, hors verrou *(développement)*

- **Contexte** : la consigne commune met la campagne de mutations sous le verrou de gate ; le verrou était tenu par
  d'autres agents, et chacune des treize mutations y aurait attendu son tour (une première baseline a attendu plus de
  dix minutes).
- **Retenu** : chaque mutation est rejouée par un **gate ciblé** — les binaires gardiens nommés au Dev Agent Record
  (27 tests backend, le fichier Vitest du composant, 36 tests), sous `mem-guard` —, ce que `docs/agents-orchestration.md`
  § 5 laisse hors verrou. Les gates complets et l'E2E complet, eux, ont tourné sous le verrou.
- **Écartées** : (a) attendre le verrou pour chaque mutation — treize attentes pour des runs de quelques secondes ;
  (b) un script qui enchaîne les treize mutations sous un seul verrou — il aurait fallu muter le code par script,
  ce que la consigne de l'orchestrateur interdit (outil Edit seul).
- **Réversible** : oui (rejouer la campagne sous verrou).

### C-29-6-21 — La phrase de la 29-2 sur le rejet est réécrite sous la réserve de #670 *(développement)*

- **Contexte** : au § *Algorithme de matching*, la 29-2 écrivait « puis rejetez la transaction […] — sans quoi elle
  reste en attente, et pourrait être passée une seconde fois », ce qui laisse entendre que le rejet la sort de
  l'attente ; la zone partagée de la fiche prescrit d'y appliquer la réserve de #670 (C-29-6-17).
- **Retenu** : « puis rejetez la transaction […], pour qu'elle ne soit pas passée une seconde fois », et une phrase
  commune aux deux contournements (paiement tardif, trop-perçu) : la transaction rejetée reste en attente et empêche
  la clôture du compte bancaire.
- **Écartée** : laisser la phrase de la 29-2 — elle contredirait la phrase neuve de la 29-6 dans le même paragraphe.
- **Réversible** : oui ; la 29-8 (#670) réécrira de toute façon ces gestes.

### C-29-6-22 — Le montant du spec E2E est tiré entre 90 000 et 99 000 CHF *(développement)*

- **Contexte** : AC10 demande un montant hors de la plage des factures des autres specs. Relevé au développement : les
  plus grands montants des specs sont de l'ordre de 1 500 CHF (`open-items.spec.ts` tire jusqu'à ~9 000).
- **Retenu** : `9 000 000 + (Date.now() % 900 000)` centimes. Contrepartie et références uniques de préfixe `REJ296`.
- **Réversible** : oui.

### C-29-6-23 — Le doc-comment du test de tolérance existant est corrigé, son code non *(développement)*

- **Contexte** : la fiche veut `find_unpaid_invoices_filters_amount_within_tolerance` « vert sans modification » ; son
  doc-comment disait « tolérance amount ± 0.05 CHF », devenu faux (le grep de propagation l'a trouvé).
- **Retenu** : le doc-comment dit la tolérance du paiement partiel et renvoie au test neuf du trop-perçu ; le code du
  test est intact, et il reste vert.
- **Écartée** : laisser le commentaire faux pour respecter la lettre de « sans modification ».
- **Réversible** : oui.

### C-29-6-24 — Une facture trop-perçue de quelques centimes qui porte un indice masque la règle *(revue de code P1, E1)*

- **Contexte** : E1 (MEDIUM, lentille E) — la story retire des candidates la facture que le paiement dépasse de 1 à
  5 centimes. Avant elle, cette facture, dès qu'elle portait un indice (référence liée à son numéro, contrepartie liée
  au nom de son client), était candidate : de rang 0 (Story 29-3), elle passait devant la règle d'affectation ou,
  à score ≥ 0,5, en empêchait le calcul — *Accepter* était refusé en trop-perçu, rien ne s'écrivait. Sans elle, la
  règle devenait `candidates[0]`, et *Accepter* passait en produit le paiement d'un client identifié, la facture
  restant due : un faux rattachement muet, créé par la story. **Décision de l'orchestrateur** : le corriger dans la
  story, la ligne restant « Aucune correspondance ». Établi au code avant d'écrire : `propose_matches` écarte une
  facture de score total nul (`matching.rs:93`), et un reste trop-perçu n'a jamais de score de montant (arrondi au
  centime ≠ paiement) ; la facture ne masquait donc la règle, avant la story, **que si elle portait un indice**.
- **Retenu** : `find_invoices_slightly_overpaid_for_window` (dépôt) lit les factures de la même fenêtre dont le reste dû
  est dans `[tx − 0.05 ; least_due_without_overpayment(tx)[` — exactement celles que l'ancienne borne basse chargeait
  et que la story écarte ; même requête que `find_unpaid_invoices_for_window` (fragment commun
  `unpaid_invoices_in_window_sql`, seule la clause `HAVING` diffère), ordre d'imputation, `LIMIT 50` propre.
  `get_proposals` ne la lit que s'il existe une règle active (règles lues avant le Pass 2), note ces factures par
  `propose_matches` comme les candidates, et n'applique pas la règle si l'une est retenue. Elles ne sont jamais
  proposées. Tests : `a_rule_never_replaces_an_invoice_the_payment_slightly_exceeds` (100.01, 100.05 : aucune
  candidate ; 100.00 : la facture ; 100.06 : la règle), `the_rule_guard_needs_an_invoice_bearing_a_signal` (sans
  indice : la règle ; avec la référence : rien) — rouges sans le correctif (`29-6-p1-red.log`) ; dépôt :
  `find_invoices_slightly_overpaid_returns_what_the_lower_bound_excludes`. Mutations M14 à M17, tuées.
- **Écartées** : (a) un drapeau d'**existence** seul (« une facture trop-perçue existe ») — plus simple d'une requête
  `EXISTS`, mais il masquerait la règle pour **toute** facture d'un autre client du même montant, sans indice, ce que
  la présence de la facture ne faisait pas avant la story : une régression pour les règles légitimes (remboursements,
  virements internes de même montant qu'une facture ouverte) ; le test « sans indice » rougit sous cette forme
  (M15). (b) Recharger les trop-perçus dans la même requête et filtrer en Rust — rouvre le défaut du `LIMIT 50`
  (C-29-6-3). (c) Masquer la règle seulement quand les candidates sont vides — équivalent aujourd'hui (toute candidate
  est de rang 0 et passe devant la règle), mais repose sur l'inatteignabilité du rang 2 que la 29-3 garde pour
  l'avenir. (d) Refuser la règle à l'acceptation — un appel direct choisit la règle en connaissance de cause, et le
  défaut est dans ce que l'écran propose.
- **Écart d'exactitude, assumé** : quand une facture valide **faible** côtoie une facture trop-perçue désignée, la règle,
  qui figurait avant la story en `candidates[1]` (invisible à l'écran, jamais acceptée par lui), n'y figure plus. Coût :
  une requête de plus par transaction créditrice en CHF, seulement s'il existe une règle active ; ces factures
  ajoutent leurs clients au chargement groupé des contacts (`MAX_IN_CLAUSE_IDS`, 500, inchangé — avant la story, elles
  y figuraient déjà, comme candidates).
- **Réversible** : oui (une fonction de dépôt, une condition de la route).

### C-29-6-25 — Le libellé accessible de la case et l'annonce de l'indication sont écrits comme limites *(revue de code P1, B3 = E3)*

- **Contexte** : B3 = E3 (LOW) — la case de chaque ligne n'a pas de libellé accessible (défaut antérieur, étendu par la
  story aux lignes sans proposition) et l'indication apparaît au cochage sans région `aria-live`.
- **Retenu** : écrits comme **limites**, non corrigés. Aucun des deux n'est une ligne : un libellé de case utile nomme
  la transaction (date, montant, contrepartie) et demande une clé i18n à variables dans les quatre locales, plus le
  `sitesTotal` ; une annonce fiable demande une région présente **avant** le changement (un `role="status"` posé sur
  le paragraphe inséré par le `{#if}` n'est pas annoncé de façon fiable), donc un paragraphe toujours rendu, ce qui
  change le contrat du `data-testid` qu'éprouvent les tests Vitest et le spec E2E. Issue à ouvrir par l'orchestrateur
  (accessibilité de la table de réconciliation : libellé des cases, annonce de l'indication), P4.
- **Écartée** : un `role="status"` d'une ligne, qui donnerait l'apparence d'une correction sans en garantir l'effet.
- **Réversible** : oui.

### C-29-6-26 — La phrase de priorité de l'écran reste simplifiée *(revue de code P1, propagation d'E1)*

- **Contexte** : le grep de propagation d'E1 trouve la phrase de la 29-3 au-dessus de la liste, « la facture la mieux
  notée s'il y en a une, sinon la règle d'affectation qui correspond » (quatre locales) ; elle ne dit pas l'exception
  neuve (une facture trop-perçue désignée masque la règle).
- **Retenu** : laissée telle quelle. Elle décrit ce qui est **affiché** : dans le cas neuf, rien ne l'est, la ligne dit
  « Aucune correspondance » ; elle ne promet donc rien de faux à l'écran. L'exception est écrite au manuel (§
  *Algorithme de matching*, liste de l'écran, § *Règles d'affectation*, FAQ), à `api-external.md` et au CHANGELOG.
  Les phrases du manuel qui disaient la règle proposée « quand aucune facture n'est candidate » portent l'exception.
- **Écartée** : réécrire la clé dans les quatre locales — une phrase plus longue pour un cas que la ligne dit déjà.
- **Réversible** : oui.

### C-29-6-27 — La propreté de l'arbre au lancement des gates s'écrit en clair *(revue de code P1, A2)*

- **Contexte** : A2 (LOW) — `29-6-gates-head.txt` ne contenait qu'un hash ; le script y ajoutait la sortie de
  `git status --short`, vide pour un arbre propre, ce que rien ne disait.
- **Retenu** : le script des gates de la remédiation écrit « arbre propre (git status --short vide) » quand la sortie est
  vide (`29-6-p1-gates-head.txt`). L'entrée du Dev Agent Record sur `29-6-gates-head.txt` est annotée, non réécrite.
- **Réversible** : oui.

[#665]: https://github.com/guycorbaz/kesh/issues/665
[#670]: https://github.com/guycorbaz/kesh/issues/670

---

## Story 29-8 — Une transaction réglée depuis la fiche se rattache à son règlement ([#670])

*Registre propre à la story, à verser **par union** au registre de l'epic (`epic-29-choix-autonomes.md`). Fiche :
`29-8-transaction-reglee-ailleurs.md`. Issue : #670. Relevés sur `fcd2ae75`.*

---

### C-29-8-1 — Les prémisses de #670 sont exactes

- **Contexte** : #670 affirme qu'une transaction réglée depuis la fiche, puis rejetée, reste `pending` et bloque à
  jamais la clôture du compte bancaire.
- **Retenu** : établies au code une à une (fiche, § *Les prémisses*) — toutes exactes. Aucune correction à porter à
  l'issue.
- **Réversible** : sans objet.

### C-29-8-2 — Aucune migration : `matched_entry_id` et `reconciled` suffisent

- **Contexte** : rattacher demande un lien transaction → écriture.
- **Retenu** : la colonne `bank_transactions.matched_entry_id` et le statut `reconciled` existent depuis la Story 8-1b ;
  le rattachement les pose comme les cinq autres chemins.
- **Écartée** : une colonne d'origine du lien (« rattachée » contre « créée ») — elle seule permettrait un « détacher
  sans annuler » (L1), au prix d'une migration ; hors périmètre minimal.
- **Réversible** : oui (une migration ADD COLUMN nullable plus tard).

### C-29-8-3 — Piste (a), restreinte aux règlements clients par virement

- **Contexte** : #670 propose (a) rattacher, (b) choisir une facture à la réconciliation (#666), (c) un statut
  « écartée ».
- **Retenu** : (a), restreinte aux lignes `invoice_settlements` de type `bank_transfer` : l'état produit est celui
  d'une proposition « facture » acceptée, que les gardes en aval qui lisent le lien traitent déjà (rang 3 de
  l'annulation, par le socle de la contre-passation ; suppression d'import ; *Annuler le rapprochement*).
  *Amendé en validation P1 (R2/F4)* : la dévalidation ne lit pas ce lien — son motif 5 vise l'écriture de vente ; elle
  est refusée au motif 1 (`Settled`), avant comme après le rattachement.
- **Écartées** : (b) ne couvre pas la 29-6 (trop-perçu refusé par `classify_payment`) ni les règlements déjà saisis,
  et exige une seconde borne (#548) ; (c) migration breaking et faux rattachement **muet** (une transaction jamais
  comptabilisée sortirait des contrôles de clôture) ; (a) généralisée à toute écriture — *Annuler le rapprochement*
  contre-passerait une écriture saisie au journal (`ReconciliationKind::Entry`) : il faudrait d'abord C-29-8-2
  écartée.
- **Réversible** : oui (élargir plus tard, après une colonne d'origine).

### C-29-8-4 — Les quatre conditions du rattachement

- **Contexte** : un faux rattachement doit être impossible ou visible.
- **Retenu** : transaction `pending` crédit CHF ; règlement de la société, `bank_transfer`, **même compte bancaire** ;
  écriture rapprochée d'aucune transaction ; `M > 0` et écart `0 ≤ t.amount − M ≤ 0,05`, `M` lu sur le compte du grand
  livre actuel du compte bancaire (*`M > 0` ajouté en validation P1*, C-29-8-18).
- **Écartée** : comparer `invoice_settlements.amount` (montant réglé, qui peut différer du payé d'un demi-centime
  d'arrondi) — la ligne de banque est l'argent.
- **Réversible** : oui.

### C-29-8-5 — Un écart de 0 à 5 centimes, dans un seul sens, affiché

- **Contexte** : le contournement de la 29-6 enregistre le **montant dû** ; la transaction le dépasse de 1 à
  5 centimes. Un rattachement à montant strictement égal laisserait ce cas bloqué.
- **Retenu** : `0 ≤ écart ≤ AMOUNT_TOLERANCE_HUNDREDTHS` (constante existante) ; l'écart s'affiche avant confirmation
  et s'écrit à l'audit. Négatif exclu (argent aux livres que la banque n'a pas reçu).
- **Écartée** : égalité stricte (29-6 non couverte) ; tolérance symétrique (aucun contournement ne la produit).
- **Réversible** : oui.

### C-29-8-6 — Pas de fenêtre de dates : la proximité trie et s'affiche

- **Contexte** : « dates proches — à établir ».
- **Retenu** : le dialogue de règlement préremplit la date au jour (`SettleInvoiceDialog.svelte:76-90`) et Guy importe
  un relevé par mois : une borne refuserait le cas nominal. Tri par écart puis jours d'écart ; colonne « jours ».
- **Écartée** : une borne de ± 30 ou ± 60 jours.
- **Réversible** : oui.

### C-29-8-7 — Un geste humain, sans présélection, absent des propositions

- **Contexte** : règle *Un appariement automatique propose, il ne crée jamais*.
- **Retenu** : liste de candidats, aucun présélectionné, confirmation explicite ; `GET /reconciliation/proposals`
  inchangé.
- **Écartée** : proposer le rattachement parmi les candidates de la liste (nouveau type de candidate, acceptation en
  lot).
- **Réversible** : oui.

### C-29-8-8 — Deux routes unitaires, sous `/reconciliation/transactions/{id}/`

- **Contexte** : forme de l'API.
- **Retenu** : `GET …/link-candidates`, `POST …/link` `{ settlementId }` ; le compte bancaire se lit sur la
  transaction (comme `…/cancel` et `…/unreject`). Pas un lot : le pattern batch ne s'applique pas.
- **Écartée** : un corps `{ bankAccountId, bankTransactionId, … }` comme `/manual` (redondant avec l'identifiant de
  chemin).
- **Réversible** : oui avant publication.

### C-29-8-9 — Une seule définition du « rattachable »

- **Contexte** : la liste et le refus ne doivent pas diverger.
- **Retenu** : condition 1 en fonction pure de `kesh-db` ; conditions 2 à 4 en **une** requête, rejouée par le `POST`
  restreinte à `s.id`. Mutations M1 à M19 (M14 à M19 ajoutées en validation P1).
- **Écartée** : un classifieur Rust des motifs à côté de la requête (deux définitions).
- **Réversible** : oui.

### C-29-8-10 — Un seul code de refus, `409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE`

- **Contexte** : motifs multiples (compte, montant, déjà rapproché, transaction non créditrice).
- **Retenu** : un code, `details.bankTransactionId` et `details.settlementId` ; l'écran n'offre que des candidats, le
  refus ne se voit que par une course ou un appel direct — conduite : relire la liste. Les 404 restent distincts
  (anti-énumération).
- **Écartée** : un `details.reason` par condition — il exigerait le classifieur écarté en C-29-8-9.
- **Réversible** : oui (ajouter un `reason` est compatible).

### C-29-8-11 — Rejouée sur interblocage, classée `SansEcritureAuJournal`

- **Contexte** : la route prend des verrous de ligne sur `invoice_settlements` et `journal_entries`.
- **Retenu** : enveloppe `retry_app_on_deadlock`, comme `post_manual` ; registre `Traced, SansEcritureAuJournal` (le
  registre n'interdit pas l'enveloppe, point (vi)).
- **Écartée** : non rejouée comme `…/unreject` (qui ne verrouille que sa transaction).
- **Réversible** : oui.

### C-29-8-12 — Ordre des verrous imposé, vue née après le dernier

- **Contexte** : B5 de la rétrospective 28 (naissance de la vue).
- **Retenu** (*amendé en validation P1, F1/F6*) : verrou nommé → `bank_transactions` (`t`) → `invoice_settlements`
  (`s`) → `journal_entries` (`je`), en **deux instructions** pour `s` puis `je` (l'ordre est écrit, non laissé à
  l'optimiseur d'un `JOIN`), sans l'exercice → `ensure_open_in_tx` (première lecture ordinaire) → requête des
  conditions. Ordre `s → je` aligné sur `cancel_settlement_in_tx` ; les lignes `invoices`, `bank_accounts` et
  `fiscal_years` ne sont jamais prises. Table *Absence de cycle* aux Dev Notes, à recopier au Dev Agent Record.
- **Écartée** : la lecture courante `matched_entry_id = ? FOR UPDATE` de la première rédaction (C-29-8-17).
- **Réversible** : oui.

### C-29-8-13 — *Annuler le rapprochement* d'une transaction rattachée annule le règlement (L1)

- **Contexte** : sans colonne d'origine (C-29-8-2), l'état est indiscernable d'une proposition acceptée.
- **Retenu** : comportement existant gardé et écrit au manuel ; visible (contre-passation, facture rouverte),
  réparable, sans erreur comptable. Évolution tracée par **#672** (« détacher sans annuler »).
- **Écartée** : refuser l'annulation d'une transaction rattachée (indiscernable sans migration).
- **Réversible** : oui.

### C-29-8-14 — Le bouton sur la ligne des propositions, pas au détail de l'import

- **Contexte** : où offrir le geste.
- **Retenu** : bouton de ligne *Rattacher à un règlement* sur les lignes crédit CHF de `ReconciliationProposals` ;
  une transaction rejetée passe d'abord par *Annuler le rejet* (l'API admet une rejetée, comme `/manual`).
- **Écartée** : un second point d'entrée au détail de l'import (deux écrans, deux tests E2E).
- **Réversible** : oui.

### C-29-8-15 — « Rattachez » remplace « rejetez » dans les contournements de la 29-2 et de la 29-6

- **Contexte** : les fiches 29-2 (C-29-2-8) et 29-6 (AC7) prescrivent « régler depuis la fiche, puis rejeter » ;
  même release v0.13.4.
- **Retenu** : la 29-8, développée après elles, réécrit ces phrases (manuel, CHANGELOG, `api-external.md` s'il y a
  lieu) et retire les incises « (#670) » devenues fausses. L'indication de l'écran de la 29-6 reste (elle parle des
  boutons de lot).
  *Annoté le 2026-10-14 (validation P2, R3) : la 29-6, à sa validation P2, a réécrit elle-même la FAQ (« Kesh ne relie
  pas la transaction … ; rejetez-la »), complété `:2050` et rédigé l'indication « ne peut pas être acceptée » ; au
  manuel, ses incises ne portent **aucun** « 670 » (C-29-6-17) — seule son entrée de CHANGELOG le porte. La fiche 29-8
  (AC6) cite désormais ces textes tels qu'ils sont ; le retrait des incises vaut pour l'incise « rejetée, elle reste en
  attente … » de `:2029` et de la FAQ, et pour « ([#670]) » de l'entrée de CHANGELOG de la 29-6.*
  *Annoté le 2026-10-14 (validation P3, R1) : la fiche ne recopie plus les phrases prescrites par les fiches voisines
  (elles bougent : la 29-2 a supprimé la phrase de `:1305`, et réécrit `:2031`, `CHANGELOG.md:15`; la 29-5 prescrit le
  même conseil, `29-5-departage-exigibilite.md:326-327`). L'inventaire des conseils de rejet se fait au rebase, par le
  symptôme — voir C-29-8-29.*
- **Écartée** : laisser les deux gestes au manuel (le rejet ne règle pas #670).
- **Réversible** : oui.

### C-29-8-16 — Pas de découpage ; inventaire clos des textes « cinq chemins »

- **Contexte** : règle de découpage (> 5 modules).
- **Retenu** : quatre modules de code (`kesh-db`, `kesh-api`, `kesh-i18n`, `frontend/…/reconciliation`) plus la
  documentation : une seule story. Les cinq textes qui supposent « une écriture créée par le rapprochement » sont
  inventoriés (AC7) et réécrits.
- **Réversible** : oui.

---

*Choix C-29-8-17 à C-29-8-24 : remédiation de la validation P1 (2026-10-14), lentilles R (0/0/4/7) et F (0/1/3/5).*

### C-29-8-17 — Aucune lecture verrouillante sur `matched_entry_id` ; l'invariant tient par le verrou du règlement

- **Contexte** : l'étape 4.3 d'origine (`SELECT … WHERE matched_entry_id = ? FOR UPDATE`) devait lire « au courant »
  la condition 3. Validation P1, F1 (HIGH) : sur un index secondaire non unique, un `FOR UPDATE` sans ligne trouvée
  pose un verrou d'intervalle ; deux rattachements simultanés dont les écritures tombent dans l'intervalle terminal
  s'interbloquent à l'`UPDATE` (verrou d'insertion). Et elle est redondante.
- **Retenu** : étape retirée. L'invariant « une écriture, une transaction » se tient par la lecture ordinaire des
  conditions **sous le verrou de ligne de `s`** : deux rattachements du même règlement visent deux transactions du
  même compte (condition 2) ; le second attend `s` jusqu'au **commit** du premier (le verrou nommé, lui, est relâché
  avant le commit, `mutex.rs:126-132` et `conclude_locked_attempt`) ; sa vue naît après l'attente et voit le lien.
  Les cinq autres chemins ne lient que des écritures qu'ils créent ; le seul autre écrivain de la colonne la remet à
  `NULL` sous le même verrou de `s`. `GET_LOCK` ne fait pas naître la vue (aucune table lue ; fiche 28-4:961). Test
  T17 (course de double rattachement), mutation M14 (requête des conditions avant le verrou de `s`).
  *Annoté le 2026-10-14 (validation P2, R1 = F1 ; C-29-8-25) : T17 n'observe plus l'attente — il n'asserte que le
  résultat d'une course ; l'ordre d'acquisition et la naissance de la vue sont renvoyés à la revue de code, et M14 n'est
  tuée que si les deux requêtes se chevauchent.*
- **Écartées** : garder la lecture en la forçant sur l'index (`FORCE INDEX`) avec un test `EXPLAIN` — le cycle
  d'intervalle demeure ; une contrainte `UNIQUE (matched_entry_id)` — migration, et hors périmètre minimal.
- **Réversible** : oui.

### C-29-8-18 — `M > 0` dans la condition 4

- **Contexte** : validation P1, R3 = F2 : avec `M = 0` (règlement sur un ancien compte lié), tout crédit de 1 à
  5 centimes rendait le règlement candidat à « 0.00 ».
- **Retenu** : `M > 0` (un règlement par virement débite toujours la banque, `settle_invoice` refuse `amount <= 0`) ;
  table dérivée par écriture en jointure interne, `HAVING M > 0`. Test T18 (ancien compte lié, crédit de 0.03 et
  crédit égal), mutations M15 et M16.
  *Annoté le 2026-10-14 (validation P2, R2 = F2) : dans la forme retenue, c'est la **jointure interne** qui écarte
  l'écriture sans ligne sur le compte lié ; le `HAVING M > 0` ne décide que d'une somme ≤ 0, qu'aucun état atteignable
  ne produit — M15 est déclarée **équivalente** (fiche, AC8), non tuée par T18 (a).*
  *Annoté le 2026-10-14 (validation P3, R4 = F1) : la raison est rectifiée. « Aucun état atteignable » était faux à la
  lettre — le refus du lien de la banque à un compte de créance (`repositories/bank_accounts.rs:1273-1288`, et non
  `routes/bank_accounts.rs:426-433`, qui est le contrôle du type de compte) ne compare qu'aux comptes de créance des
  réglages **actuels** : le compte de créance d'une vente antérieure peut être lié, `M < 0`. Le `HAVING` ne le distingue
  que d'un règlement de moins de 5 centimes (`écart > 0,05` sinon) : M15 est une **survivante assumée**, équivalente
  sauf au centime, sans test.*
- **Écartée** : `COALESCE(…, 0)` sans condition (laisse passer `M = 0`).
- **Réversible** : oui.

### C-29-8-19 — `ensure_open_in_tx` dans la transaction, après le dernier verrou

- **Contexte** : validation P1, R9/F8 : un pré-vol seul se classe `Invariant("I1", …)`, non « gardé » ; le
  précédent `unreject_once` lit la garde dans la transaction.
- **Retenu** : pré-vol **et** `bank_accounts::ensure_open_in_tx` à l'étape 4.3, première lecture ordinaire, après le
  verrou de `je` ; statut `Gardee` au `REGISTRE` de `bank_account_closed_inventory.rs`. Test T13 (b) (clôture posée
  pendant l'attente du verrou nommé), mutation M19. Zone partagée avec la 29-4 (`FENETRE`) : la seconde mergée ajoute
  la ligne.
- **Écartée** : pré-vol seul classé `Invariant("I1", …)` — sûr (I1 tient), mais `link_in_tx` serait la seule
  fonction de rapprochement sans garde en transaction parmi celles qui en ont une voisine.
- **Réversible** : oui.

### C-29-8-20 — Compte bancaire archivé refusé ; compte du grand livre inactif admis

- **Contexte** : validation P1, R10.5 : `post_manual` refuse les deux ; la fiche n'en disait rien.
- **Retenu** : archivé → `AppError::BankAccountNotFound` au `GET` et au `POST`, comme `post_manual` ; compte du
  grand livre inactif **admis** (le rattachement n'y écrit rien). Test T20.
- **Écartée** : refuser l'inactif (`412`) — il protège une écriture que le rattachement ne passe pas.
- **Réversible** : oui.

### C-29-8-21 — Exercice clos ou période verrouillée : le rattachement passe

- **Contexte** : validation P1, R10.4.
- **Retenu** : admis (aucune écriture au journal ; le verrou de `je` ne prend pas l'exercice) ; la conséquence —
  *Annuler le rapprochement* ensuite refusé `FISCAL_YEAR_CLOSED` — écrite au manuel. Test T19.
- **Écartée** : refuser (un paiement de décembre rapproché en janvier, une fois décembre clos, resterait bloqué).
- **Réversible** : oui.

### C-29-8-22 — Sérialisation, tolérance, date de paiement : une seule définition chacune

- **Contexte** : validation P1, R10.1 à R10.3 et F9 (types).
- **Retenu** : conditions calculées en décimal exact dans la requête ; sortie JSON et audit `rescale(2)` (patron
  `routes/reconciliation.rs:4274-4283`) ; tolérance liée en `Decimal`, passée en paramètre à `kesh-db` (patron
  `find_unpaid_invoices_for_window`), la constante restant privée à `routes/reconciliation.rs` ; `payment_date(t)`
  calculée en Rust et liée en paramètre, la requête triant sur `ABS(DATEDIFF(s.settled_on, ?))`.
- **Écartées** : rendre la constante publique dans `kesh-db` (deux crates la verraient, une seule la définit
  aujourd'hui) ; réécrire `COALESCE(value_date, booking_date)` en SQL (seconde définition de la date de paiement) ;
  trier en Rust après une requête sans `LIMIT`.
- **Réversible** : oui.

### C-29-8-23 — Angle mort L6 : le paiement qui inclut les frais de rappel

- **Contexte** : validation P1, R4 : le manuel prescrit d'enregistrer le seul montant de la QR-facture quand le client
  paie « reste dû et frais » ; la transaction dépasse alors le règlement de plus de 5 centimes.
- **Retenu** : angle mort écrit (Dev Notes L6, manuel `:1308`, CHANGELOG), renvoyé à #671 (commentaire posé par
  l'orchestrateur) et à #401.
- **Écartée** : élargir la tolérance ou admettre tout dépassement — hors périmètre minimal, et un dépassement franc
  doit se justifier par une écriture que le rattachement ne passe pas.
- **Réversible** : oui.

### C-29-8-24 — Un refus `RECONCILIATION_SETTLEMENT_NOT_LINKABLE` recharge la liste de la modal

- **Contexte** : validation P1, R11 : le message dit « relire la liste », la modal ouverte ne la rechargeait pas.
- **Retenu** : sur ce `409`, la modal recharge `getLinkCandidates` et remet le choix à vide ; l'échec du chargement
  s'affiche. Vitest.
- **Écartée** : fermer la modal (l'utilisateur perdrait le contexte de la transaction).
- **Réversible** : oui.

---

*Choix C-29-8-25 à C-29-8-28 : remédiation de la validation P2 (2026-10-14), lentilles R (0/0/5/5) et F (0/0/5/6). Les
trois premiers sont des **décisions de l'orchestrateur** (« retirer plutôt qu'ajouter »), consignées comme telles.*

### C-29-8-25 — L'observation de l'attente est retirée de T16 et T17 *(décision de l'orchestrateur)*

- **Contexte** : validation P2, R1 = F1 (MEDIUM, né de la remédiation P1) : T17 prescrivait d'observer le `POST`
  bloqué sur le verrou de ligne par `information_schema.INNODB_TRX`, que l'utilisateur de dev (`kesh`, sans privilège
  `PROCESS`, `scripts/mariadb-init/01-dev-grants.sql`) ne peut pas lire ; T16 renvoyait à la même observation.
- **Retenu** : **retirer** l'observation, ne la remplacer par **aucune** autre. T17 asserte le seul résultat d'une
  course réelle (deux rattachements concurrents du même règlement, `tokio::join!` : exactement un `200`, l'autre
  `409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE`, un seul lien en base) ; T16 de même, entre l'annulation du règlement
  depuis la fiche et le rattachement (exactement l'un réussit, état final cohérent). Ce qu'ils ne prouvent pas —
  l'ordre d'acquisition, l'attente du perdant, le moment où naît la vue — est écrit à la fiche et **renvoyé à la revue
  de code** ; M10 et M14 ne sont tuées que si les requêtes se chevauchent (une survie se déclare au Dev Agent Record).
- **Écartées** : la sonde `attendre_une_requete_en_cours` sur la requête de l'étape 4.2 (a) (proposée par R1 et F1) —
  une autre forme d'observation, que l'orchestrateur a exclue ; garder `INNODB_TRX` (rouge permanent sur le poste de
  dev).
- **Réversible** : oui (une observation peut être ajoutée par une story ultérieure).
  *Annoté le 2026-10-14 (revue de code P1, E1, relevé au-dessus de LOW par l'orchestrateur) : la motivation du
  contexte est inexacte — le privilège `PROCESS` ne concerne qu'`information_schema.INNODB_TRX` ; la sonde
  `attendre_une_requete_en_cours` lit `PROCESSLIST`, que l'utilisateur `kesh` voit pour ses propres connexions (T13 (b)
  l'employait déjà). La décision est **levée** pour T17 : un test déterministe T17-bis est ajouté (C-29-8-41) ; T16 et
  T17 restent tels quels.*

### C-29-8-26 — T15 ne clôt pas le compte ; la clôture est assertée par T1 *(décision de l'orchestrateur)*

- **Contexte** : validation P2, R4 = F4 : un compte clos fait rendre `409 BANK_ACCOUNT_CLOSED` à toute annulation qui
  suit, avant ses propres motifs (`reconciliation_cancel.rs:325`, `invoice_settlements_write.rs:896`) ; un compte
  principal ne se clôt pas (`bank_accounts.rs:1016-1019`) ; `…/cancel` supprime le règlement
  (`invoice_settlements_write.rs:946`).
- **Retenu** : T15 opère sur un compte **non principal** qu'il ne clôt pas, assertions dans un ordre écrit (dévalidation
  avant ; rattachement ; refus `MATCHED_BANK_TRANSACTION` puis témoin annulé ; dévalidation après ; K2 ; `…/cancel` en
  dernier) ; le témoin est un acompte **par virement sur le même compte**. La clôture refusée puis réussie — le défaut
  de #670 — passe à **T1**, compte non principal.
- **Écartées** : clore puis rouvrir le compte dans T15 (deux gestes de plus, et la réouverture n'est pas l'objet) ;
  deux décors dans T15.
  *Annoté le 2026-10-14 (validation P3, F5) : l'exigence « compte non principal » de T15 est retirée — sa raison d'être
  (un principal ne se clôt pas) est passée à T1 ; T15 n'a plus à imposer un second compte bancaire.*
- **Réversible** : oui.

### C-29-8-27 — Le `GET` des candidats applique le pré-vol du `POST`

- **Contexte** : validation P2, R9 = F10 (LOW) : l'AC1 promettait « la liste n'offre jamais ce que le `POST`
  refuserait » sans prescrire au `GET` le compte clos ni le compte sans compte du grand livre lié.
- **Retenu** : mêmes fonctions de pré-vol au `GET` (archivé → `BankAccountNotFound`, clos → `409
  BANK_ACCOUNT_CLOSED`, non lié → `412 BANK_ACCOUNT_NOT_CONFIGURED`) ; T13 (a) asserte le `GET` aussi.
- **Écartée** : `200` et `[]` — la modal afficherait « Aucun règlement … » et tairait la cause.
  *Annoté le 2026-10-14 (validation P3, R9 = F6) : « mêmes fonctions » était inexact — aucune fonction existante ne
  rend les trois refus (`ensure_bank_account_open` ne rend que le clos, `_ => Ok(())` sinon). Le dev écrit **une**
  fonction privée `link_preflight`, appelée par les deux handlers, dans l'ordre archivé, clos, non lié, avant toute
  lecture de statut ; l'ordre des refus du `GET` est celui du `POST` (404 inexistante, archivé, clos, 412, puis 404
  rapprochée), faute de quoi un compte clos rendrait `404` au `GET` et `409` au `POST`.*
- **Réversible** : oui.

### C-29-8-28 — Angle mort L7 : règlement par compte interne sur le compte du grand livre de la banque *(décision de l'orchestrateur)*

- **Contexte** : validation P2, F3 : `settle_invoice` admet un règlement `internal_account` dont le compte est celui du
  grand livre de la banque ; son écriture débite la banque, mais il n'est pas un virement.
- **Retenu** : non rattachable (condition 2), angle mort **L7** écrit (Dev Notes, sous-section neuve du manuel,
  CHANGELOG), même remède que L5 (annuler et refaire par virement), versé à #671 par l'orchestrateur ; **aucun test
  neuf**. Dans le même mouvement, M2 est déclarée équivalente (contrainte `chk_invoice_settlements_counterparty`),
  M15 (jointure interne) et M17 (filtre société de l'étape 4.2 (a)) aussi, sans décor pour les tuer ; M1 reste tuée
  par T6, au prix d'une ligne de montage (compte du grand livre partagé).
- **Écartées** : admettre ce règlement au rattachement (élargit la condition 2 hors du périmètre minimal) ; construire
  des décors forgés en SQL pour tuer M2, M15, M17.
- **Réversible** : oui.

### C-29-8-29 — Inventaire des conseils de rejet par le symptôme, au rebase ; retrait du contrôle négatif « tout seul. » *(décision de l'orchestrateur)*

- **Contexte** : validation P3, R1 (MEDIUM) : le tableau de l'AC6 décrivait une 29-2 périmée (sa fiche réelle supprime
  la phrase de `:1305`, réécrit `:2031` en « en rejetant ensuite la transaction », et son entrée de CHANGELOG, ligne 15,
  dit « puis rejetez la transaction ») et ignorait la 29-5 (`29-5-departage-exigibilite.md:326-327`, même conseil).
  R2 (MEDIUM) : le contrôle négatif « tout seul. » = 0 contredisait `:1198`, qui garde la phrase.
- **Retenu** : la fiche cesse de recopier les phrases des fiches voisines. L'AC6 prescrit, à l'étape 0 et **après** rebase
  sur les 29-2, 29-5 et 29-6 mergées, `grep -nE 'rejet[a-z]*( ensuite)?( la transaction|-la)'
  docs/manual/fr/user-manual.tex CHANGELOG.md` et toute forme voisine ; chaque conseil de rejeter une transaction réglée
  depuis la fiche de la facture est réécrit en « rattachez », les autres occurrences sont triées et écrites. Le tableau du
  manuel ne garde que les sites propres à la 29-8 (glossaire, tuile d'écart, introduction, FAQ, `:1198`, `:1308`,
  `:2076`, `:2050`) et renvoie au grep pour `:2029` et `:1305`. La 29-5 entre aux zones partagées. Le contrôle de
  pré-release de la v0.13.4 (fichier d'epic) doit rendre **0 conseil de rejet** après la 29-8. Le contrôle négatif
  « tout seul. » est **retiré** (il rougirait sur une implémentation fidèle) ; il reste au relevé « ce qui doit encore y
  être ».
- **Écartées** : mettre à jour le tableau d'après la 29-2 réelle (il se périmerait au prochain commit d'une fiche
  voisine, ce que la validation P2 avait déjà payé pour la 29-6) ; durcir « tout seul. » en exigeant la suppression de
  la phrase (elle reste vraie).
- **Réversible** : oui.

## Choix du développement (2026-10-14, Opus 5.5)

### C-29-8-30 — Montage des tests d'intégration : la transaction par le dépôt, non par un CAMT

- **Contexte** : l'AC8 dit « transaction par import CAMT de fixture synthétique ».
- **Retenu** : dans `reconciliation_link_e2e.rs`, la transaction est posée par `bank_imports::create_with_transactions`,
  la fonction que la route d'import appelle — patron de `bank_accounts_closed_e2e.rs` (C-28-4-26) ; facture, validation
  et règlement passent par les routes. Le geste éprouvé est le rattachement, pas l'analyse d'un relevé ; l'E2E
  Playwright (AC9), lui, importe un vrai fichier CAMT.
- **Écartées** : un CAMT par test (vingt fixtures réécrites, sans rien prouver de plus du rattachement).
- **Réversible** : oui.

### C-29-8-31 — Le compte du grand livre lié est lu au pré-vol, comme `post_manual`

- **Contexte** : la condition 4 lit `M` sur le compte du grand livre **actuel** du compte bancaire ; la fiche ne dit pas
  où ce compte est lu au `POST`.
- **Retenu** : `link_preflight` le lit sur le pool (refus `412` s'il manque) et le passe à `link_in_tx`
  (`LinkRequest::bank_ledger_account_id`), comme `post_manual` passe `bank_ledger_account_id` à sa tentative. Un `PATCH`
  du lien entre le pré-vol et la transaction ferait lire `M` sur l'ancien compte — qui porte bien la ligne de banque du
  règlement s'il y fut passé : sans erreur comptable.
- **Écartées** : relire `journal_account_id` dans la transaction (une lecture ordinaire de plus, à placer après le
  dernier verrou, pour une course sans effet comptable) ; joindre `bank_accounts` dans la requête (la table dérivée ne
  pourrait plus se restreindre au compte).
- **Réversible** : oui.

### C-29-8-32 — Après un rattachement, le bilan dit « 1 opération(s) réussie(s). »

- **Contexte** : l'AC4 dit « compteur 1 opération(s) réussie(s). (même bilan que l'affectation manuelle,
  `clearBatchReport` d'abord) » et l'AC9 attend ce compteur ; or l'affectation manuelle **efface** le bilan sans afficher
  de compteur (`onManualSuccess`, test « efface le bilan du lot précédent »).
- **Retenu** : `onLinkSuccess` appelle `clearBatchReport()` puis pose `lastSuccessCount = 1` — le compteur que les deux
  AC décrivent ; l'affectation manuelle et l'éclatement restent inchangés.
- **Écartées** : aligner sur l'affectation manuelle (aucun compteur : contredit l'AC9) ; ajouter le compteur aux deux
  autres gestes (hors périmètre).
- **Réversible** : oui.

### C-29-8-33 — CHANGELOG : les limites dans l'entrée, aucune entrée « Modifié »

- **Contexte** : l'AC6 demande *Ajouté*, *Modifié* (le contournement n'est plus « rejeter ») et *Limites connues*.
- **Retenu** : une entrée *Ajouté* qui porte ses limites connues (L1 à L7, `#671`, `#672`, `#401`) ; les entrées des
  29-2 et 29-6 de la **même** section `[0.13.4] — Non publié` réécrites en place en « rattachez ». Le conseil de rejet
  n'a jamais été publié : une entrée *Modifié* annoncerait un changement qu'aucun utilisateur n'a vu. Le dépôt n'a pas
  de rubrique « Limites connues ».
- **Écartées** : une rubrique neuve « Limites connues » (hors du format Keep a Changelog suivi par le fichier).
- **Réversible** : oui.

### C-29-8-34 — Le pré-vol commun s'appelle `link_preflight` ; statut du registre adapté

- **Contexte** : l'AC2 prescrit le statut `Gardee("pré-vol (ensure_bank_account_open) et …")`, forme de
  `unreject_once` ; or le pré-vol du rattachement rend trois refus (archivé, clos, non lié) qu'`ensure_bank_account_open`
  ne rend pas (validation P3, R9).
- **Retenu** : le statut nomme `link_preflight`, la fonction réellement appelée.
- **Réversible** : oui.

### C-29-8-35 — Le point (vi) d'`audit_route_registry.rs` passe de **treize** à **quatorze**

- **Contexte** : la fiche, relevée sur `fcd2ae75`, prescrivait « sept → huit » ; la 29-4a l'a depuis porté à treize.
- **Retenu** : « quatorze », et une phrase neuve pour la route, « aucune preuve dynamique » ; l'historique « Treize : dix
  avant la revue de code P1 de la 29-4a, plus ces trois » est gardé tel quel. Les quatre assertions chiffrées recomptées
  depuis le registre : 119, 113, 95, 122 et 122.
  *Annoté le 2026-10-14 (revue de code P1, A5) : lire « les **cinq** assertions chiffrées » — cinq valeurs suivent,
  comme le disent la fiche (AC2, T3) et le code.*
- **Réversible** : oui.

### C-29-8-36 — `FENETRE` : `link_in_tx` entre `Fermee`, `G` passe à onze

- **Contexte** : la 29-4a s'est mergée la première ; la 29-8, seconde, inscrit `link_in_tx` au registre `FENETRE`
  (AC2).
- **Retenu** : `Fenetre::Fermee("I1 : la transaction est pending, la clôture refuse tant qu'elle existe — comme
  l'annulation de rejet")` — la variante que le code nomme `Fermee` (le `SansFenetre` de la fiche, cf. le doc-comment de
  l'enum) ; l'anti-muet `G` passe de 10 à 11. *Au rebase sur la 29-4b (`12c64c39`)* : la réserve « sous réserve de la
  29-4b » de la première rédaction est retirée, comme la 29-4b l'a retirée de l'annulation de rejet ; son décompte de
  sept revérifications est conservé.
- **Réversible** : oui.

## Choix de la remédiation de la revue de code P1 (2026-10-14, Opus 5.5)

*Lentilles B (0/0/0/7), E (0/0/0/4), A (0/0/1/6). E1 relevé au-dessus de LOW par l'orchestrateur ; les décisions 1 à 5
du prompt de remédiation sont de l'orchestrateur, consignées comme telles.*

### C-29-8-37 — Garde de session de la modale *(décision de l'orchestrateur, B1/E3/B2)*

- **Contexte** : la modale se ferme par Échap ou par le fond pendant l'envoi (seuls les boutons sont grisés) ; une
  réponse d'une transaction précédente s'appliquait alors à la modale rouverte (message, relecture de la liste d'une
  autre transaction, fermeture sur succès). La garde de génération du chargement existait, sans test.
- **Retenu** : un compteur `session`, incrémenté par l'effet à chaque ouverture, fermeture ou changement de
  transaction ; `handleConfirm` le capture et, s'il a changé, n'écrit ni message ni relecture ni `submitting` ; l'effet
  remet `submitting` à faux et, à la fermeture, périme le chargement en vol (`loadGen++`). **Un succès périmé prévient
  le parent seulement si aucune modale n'est ouverte** (`if (!open) onSuccess()`) : le rattachement a eu lieu, la liste
  doit être relue ; une modale rouverte entre-temps n'est pas fermée. Si elle l'a été sur la **même** transaction, son
  chargement rendra « déjà rapprochée » : la liste du parent ne sera relue qu'au geste suivant — accepté, la fenêtre
  est celle d'un aller-retour HTTP. Quatre tests Vitest de session ; mutations MF1 à MF4 tuées.
- **Écartées** : bloquer la fermeture pendant l'envoi (contredit la demande d'une garde ; la modale doit rester
  fermable) ; ignorer aussi le succès périmé modale fermée (laisserait la ligne rattachée dans la liste).
- **Non prouvé** : la garde du `finally` (`if (mySession === session) submitting = false`) et le `loadGen++` de la
  fermeture n'ont pas de mutation tuée propre (MF4 couvre la remise à zéro par l'effet).
- **Réversible** : oui.

### C-29-8-38 — La modale relit sa liste sur les trois refus d'une course *(décision de l'orchestrateur, B3)*

- **Contexte** : seul `409 RECONCILIATION_SETTLEMENT_NOT_LINKABLE` relisait ; un règlement annulé depuis la fiche rend
  `404 NOT_FOUND`, une transaction rapprochée ailleurs `404 RECONCILIATION_TRANSACTION_NOT_PENDING`.
- **Retenu** : l'ensemble `RELIRE_SUR` des trois codes ; le message du refus reste affiché ; sur `NOT_PENDING`, le
  chargement relu rend lui-même le refus (`link-load-error`), *Rattacher* grisé. `NOT_FOUND` relit aussi quand c'est la
  transaction qui a disparu avec son import : le chargement rend alors `404`, sans autre effet. Mutation MF5 tuée.
- **Écartées** : fermer la modale et recharger le parent sur `NOT_PENDING` (proposé par B3 en variante ; hors de la
  décision de l'orchestrateur).
- **Réversible** : oui.

### C-29-8-39 — Le rappel de la transaction montre sa date de paiement (A7)

- **Contexte** : la modale affichait `bookingDate`, alors que `daysApart` se compte depuis la date de paiement (valeur,
  à défaut comptabilisation, AC1).
- **Retenu**, au moindre coût : `valueDate ?? bookingDate` — un seul champ, déjà rendu par `GET /proposals` ; le manuel
  le dit. Mutation MF6 tuée.
- **Écartées** : afficher les deux dates (une colonne de plus pour un écart rare, et deux dates à distinguer) ; une clé
  i18n « Date de paiement » (le rappel n'a pas de libellé aujourd'hui).
- **Réversible** : oui.

### C-29-8-40 — Angle mort déclaré : l'archivage dans la fenêtre du pré-vol (E4)

- **Contexte** : `link_preflight` refuse un compte **archivé**, mais la garde dans la transaction
  (`ensure_open_in_tx`) ne lit que `closed_on`, et l'archivage ne refuse pas les transactions en attente. Un archivage
  commité entre le pré-vol et la transaction laisse le rattachement aboutir sur un compte archivé.
- **Retenu** : **aucun code** ; angle mort déclaré ici et au Dev Agent Record. L'effet est inoffensif : la transaction
  passe rapprochée, sans écriture, sur un compte qu'un archivage aurait de toute façon laissé avec cette transaction.
  La fenêtre du lien du grand livre (C-29-8-31) reste celle qu'elle était.
- **Écartées** : lire `archived` dans `ensure_open_in_tx` (change une fonction partagée par sept revérifications pour
  un état sans effet comptable).
- **Réversible** : oui.

### C-29-8-41 — T17-bis : l'entrelacement du double rattachement, forcé et observé *(décision de l'orchestrateur, E1)*

- **Contexte** : l'invariant « un règlement ne se rattache qu'une fois » n'a pas de contrainte `UNIQUE`, et ni T16 ni
  T17 ne rougissaient à coup sûr si sa garde sautait (M10 tuée par T16 sur un run, M14 par T11 pour une autre raison).
- **Retenu** : T17-bis — une connexion `tenue` prend `invoice_settlements … FOR UPDATE` ; le `POST` de `t2` est vu en
  attente par `attendre_une_requete_en_cours(&pool, &["invoice_settlements", "FOR UPDATE"], …)` ; `tenue` lie `t1` à
  l'écriture du règlement et commite ; le `POST` doit rendre `409 NOT_LINKABLE`, un seul lien. Vu rouge sous M10 (« le
  POST n'a jamais attendu le verrou du règlement : rendu 200 ») et sous M14 (`200` au lieu de `409`), vert sinon.
- **Écartées** : une contrainte `UNIQUE` sur `matched_entry_id` (une migration, hors du périmètre d'une remédiation et
  de la décision de l'orchestrateur, qui demande un test).
- **Réversible** : oui.

---

## Story 29-9 — Quatre petits correctifs ([#649], [#655], [#646], [#678])

*Registre propre à la story, à verser **par union** au registre de l'epic (`epic-29-choix-autonomes.md`). Fiche :
`29-9-trois-petits-correctifs.md`. Issues : #649, #655, #646, #678 (#678 ajoutée par l'amendement, C-29-9-19). Relevés sur `fcd2ae75`.*

---

### C-29-9-1 — Une story pour trois correctifs, sans découpage

- **Contexte** : story ajoutée à l'Epic 29 sur décision de Guy (2026-10-14) pour que le tag v0.13.4 compte moins
  d'issues ouvertes que celui de la v0.13.3. Trois défauts P3 indépendants : la liste de la réconciliation tronquée à
  100 (#649), une bannière d'erreur périmée (#655), une suppression sans confirmation (#646).
- **Retenu** : une seule story, sur le précédent de la 28-8 (cinq correctifs). Modules au sens de la règle : le
  frontend (un paquet npm, trois zones : réconciliation, import bancaire, contacts), `kesh-api` (un test et deux
  doc-comments, aucun code de production), la documentation, les catalogues i18n — **quatre**, sous la limite de cinq.
  Les correctifs ne se lisent ni ne s'appellent l'un l'autre ; aucun ne change une règle métier ; la revue se fait
  correctif par correctif.
- **Écartée** : trois stories (trois cycles de CI et trois rebases sur le même composant que la 29-3, la 29-6 et la
  29-8, pour le même diff).
- **Réversible** : oui — un correctif fautif sort du périmètre (son issue reste ouverte) plutôt que de retenir les deux
  autres.
- *Annotation du 2026-10-14 (validation P2, R1 = F1)* : la réécriture du commentaire de module de `kesh-db`
  (C-29-9-16) ajoute un cinquième module — un commentaire, aucun code ; cinq reste sous le seuil (« plus de 5 »).

### C-29-9-2 — #649 : un seul palier, « jusqu'à 500 », et non un « Charger plus » par cent

- **Contexte** : l'API accepte déjà `limit` jusqu'à `MAX_PROPOSALS_LIMIT = 500` (`routes/reconciliation.rs:67`) ; le
  frontend envoie toujours 100 (`reconciliation.api.ts:18`). Trois gestes possibles : un bouton qui ajoute 100 à
  chaque clic, un bouton qui passe d'un coup au plafond, une vraie pagination serveur.
- **Retenu** : un bouton **Afficher aussi les plus anciennes (jusqu'à 500 en tout)**, qui passe `limit` à 500. Deux
  états seulement (100, puis 500), donc deux branches d'écran et deux tests. Le coût d'un rechargement est
  proportionnel au nombre de transactions **réellement** en attente (une requête de candidates par transaction rendue),
  pas au `limit` : un compte qui en a 150 en charge 150, comme il en chargerait 150 avec deux clics « +100 ».
- **Écartées** : « +100 par clic » (quatre clics pour atteindre le plafond, un état par palier, sans gain de coût) ;
  pagination serveur par décalage ou par curseur (signature de `find_pending_transactions_for_account` changée, donc
  `kesh-db` touché et gate complet en boucle ; un décalage se décale à chaque transaction acceptée, ce qui exige de
  recharger toutes les pages chargées — le périmètre d'une fonctionnalité, pas d'un P3).
- **Réversible** : oui (le palier est une constante ; une pagination serveur peut venir ensuite sans défaire l'écran).

### C-29-9-3 — #649 : au-delà de 500, un avis qui dit comment faire apparaître la suite, sans conseiller le rejet

- **Contexte** : au plafond, `hasMore` peut rester vrai (plus de 500 transactions en attente sur un compte). Le
  plafond est une défense anti-DoS (M9 de la 8-4) qu'on ne lève pas.
- **Retenu** : au plafond, un avis **sans bouton** : les 500 plus récentes sont affichées, le maximum de la page ; les
  plus anciennes apparaîtront à mesure que celles-ci seront acceptées, affectées manuellement ou éclatées. C'est le
  moyen d'accès, vrai au code (la liste est triée `booking_date DESC, id DESC` et ne rend que les `pending` non
  rejetées), et déjà écrit au manuel (`user-manual.tex:2057`) et à `api-external.md:594`. **Le rejet n'est pas
  nommé** : une transaction rejetée reste `pending` et bloque la clôture du compte bancaire (fiche 29-6, #670) — le
  conseiller pour « faire de la place » créerait le défaut que la 29-8 répare.
- **Écartée** : lever ou relever le plafond (le coût d'un rechargement de 1000 transactions à une requête chacune est
  ce que M9 interdit) ; une issue pour la pagination au-delà de 500 — **laissée à l'orchestrateur** (rapport) : l'écran
  dit vrai et donne un moyen, ce que #649 demande ; une pagination serait une amélioration, pas un défaut.
- **Réversible** : oui.

### C-29-9-4 — #649 : le plafond se reflète par une constante du frontend, figée par un test Rust

- **Contexte** : le frontend doit savoir qu'il est au plafond, sans quoi il afficherait un bouton sans effet (le
  serveur ramène silencieusement `limit` à 500).
- **Retenu** : `PROPOSALS_DEFAULT_LIMIT = 100` et `PROPOSALS_MAX_LIMIT = 500` exportées de `reconciliation.api.ts`
  (le paramètre par défaut de `getProposals` lit la première — DRY), doc-comment qui renvoie à
  `DEFAULT_PROPOSALS_LIMIT` / `MAX_PROPOSALS_LIMIT` ; côté Rust, doc-comments réciproques et **un test d'intégration**
  qui fige le plafond (`limit=1000` sur 501 transactions → 500 rendues, `hasMore` vrai) et le second état de l'écran
  (`limit=500` sur 150 → 150, `hasMore` faux). *Annotation du 2026-10-14 (validation P1, F8)* : le second test porte
  sur **exactement 500** transactions (`limit=500` → 500, `hasMore` faux), pour figer la frontière que 150 laissait
  libre (`>=` pour `>`) ; il s'appelle `get_proposals_limit_500_exact_has_no_more`.
- **Écartées** : un champ `limit` effectif dans la réponse (change le contrat de l'API pour un P3) ; déduire le
  plafond de la réponse (`proposals.length < limit demandé`) — juste, mais il faut un clic inutile pour le découvrir.
- **Réversible** : oui.

### C-29-9-5 — #649 : « Afficher aussi les plus anciennes » vide la sélection, comme tout rechargement

- **Contexte** : `load()` remet `selected` à vide (`ReconciliationProposals.svelte:109`), et la fiche 29-6 s'appuie
  sur cet invariant (« `proposals` ne change qu'au `load()`… une transaction ne peut pas perdre sa proposition en
  restant cochée », état (a)).
- **Retenu** : le bouton recharge par le même `load()` ; la sélection est vidée, et le manuel le dit.
- **Écartée** : conserver l'intersection de la sélection (rompt l'invariant (a) de la 29-6, pour un geste rare).
- **Réversible** : oui.

### C-29-9-6 — #649 : un seul déclencheur du rechargement, l'`$effect`

- **Contexte** : `load()` est lancé par `$effect(() => { void load(); })` (`:118-120`) ; il lit `bankAccountId`
  avant son premier `await`, donc l'effet le suit. Lire `limit` au même endroit le fera suivre aussi : un bouton qui
  poserait `limit` **puis** appellerait `load()` lancerait **deux** requêtes.
- **Retenu** : le bouton ne fait que `limit = PROPOSALS_MAX_LIMIT` ; l'effet recharge. Commentaire au site ; test qui
  exige **exactement** deux appels de `getProposals` (montage, clic). Le `{#key selectedId}` de la page
  (`routes/(app)/reconciliation/+page.svelte`) remonte le composant à chaque changement de compte : `limit` repart à
  100 — à écrire dans le commentaire, comme C-15-6-28 l'a fait pour `bankLedgerAccountId`.
- **Écartée** : `untrack` + appel explicite (deux mécanismes pour un).
- **Réversible** : oui.

### C-29-9-7 — #655, premier cas : un numéro de séquence dans `load()` du détail d'import

- **Contexte** : `bank-import/[id]/+page.svelte:47-70` n'a pas de garde : la réponse la plus lente l'emporte, échec
  comme succès.
- **Retenu** : le patron déjà en place dans `ReconciliationProposals.svelte:99-117` (`loadGen`) : `const mySeq =
  ++loadSeq` ; après l'`await`, `detail`, `errorMessage` et `loading` ne s'écrivent que si `mySeq === loadSeq`. La
  remise à `null` en tête (28-5b) reste.
  *Annotation du 2026-10-14 (validation P1, F7 et R8)* : `loading` sort de la garde — voir C-29-9-14 ; `++loadSeq`
  est la première instruction de `load()`, avant la branche « Identifiant invalide » (R7).
- **Écartée** : `AbortController` (l'`apiClient` remplace le `signal` de l'appelant par le sien, `api-client.ts:258-260` ; plus large pour le même effet).
- **Réversible** : oui.

### C-29-9-8 — #655, second cas : prémisse rectifiée — défaut latent, corrigé quand même

- **Contexte** : `bank-import/profiles/[id]/+page.svelte` ne remet jamais `errorMessage` à `null` (exact) ; mais son
  `$effect` ne se rejoue que si `page.params.id` change **sans quitter la route**, et aucun lien de l'application ne
  mène d'un profil à un autre (`git grep "bank-import/profiles/"` : la liste, séparée, et `new` → `goto` vers
  `[id]`, une autre route). Le symptôme n'est **pas atteignable** depuis l'interface au sol.
- **Retenu** : corriger quand même, à l'identique du premier cas, parce que c'est mécanique et que la route le
  permettra dès qu'un lien direct apparaîtra : en tête de l'effet `errorMessage = null`, `isLoading = true`, numéro de
  séquence. `isLoading = true` démonte le formulaire pendant la relecture, si bien que `BankProfileForm` repart du
  nouveau profil et non de l'ancien. Test par un `page` réactif (fixture `.svelte.ts`). L'orchestrateur commente #655
  (prémisse « même symptôme » → « même défaut, latent »).
- **Écartée** : sortir le second cas du périmètre (l'issue resterait ouverte pour trois lignes).
- **Réversible** : oui.
- *Annotation du 2026-10-14 (validation P1, F4)* : la prémisse « son `$effect` ne se rejoue que si `page.params.id`
  change » est **fausse au code** — l'effet dépend du signal `page.params`, que SvelteKit réassigne (objet neuf) à
  chaque navigation dans la route, même à `id` égal (`@sveltejs/kit/src/runtime/client/state.svelte.js:37`,
  `client.js:1888`). Telle quelle, la correction retenue ci-dessus (`isLoading = true` en tête) aurait démonté le
  formulaire et perdu une saisie à chaque navigation de ce genre. Le « Retenu » ci-dessus est conservé pour la trace ;
  il est **remplacé par C-29-9-13** pour la forme de l'effet. Le constat « défaut latent » tient (aucun lien au sol).
  *Précision du 2026-10-14 (validation P2, F5)* : « à chaque navigation » est trop large — à chaque navigation où
  `page_changed` est vrai (`client.js:763-772`), ce qui comprend une navigation à `id` égal qui change l'URL ou les
  données ; la conclusion ne change pas.

### C-29-9-9 — #655 : la classe n'est pas revendiquée fermée

- **Contexte** : d'autres pages peuvent relire sans séquence ou garder une erreur périmée.
- **Retenu** : la story ferme les **deux sites** que nomme #655, et la fiche ne prétend rien de plus (aucun critère ne
  ferme une classe, donc pas d'inventaire clos à tenir). Le Change Log le dira.
- **Écartée** : un inventaire des `load()` de toute l'application (hors du périmètre minimal ; une issue si un site
  atteignable est trouvé en chemin).
- **Réversible** : oui.

### C-29-9-10 — #646 : `window.confirm`, qui nomme la personne

- **Contexte** : le gestionnaire des personnes vit **dans** le dialogue d'édition d'un contact
  (`routes/(app)/contacts/+page.svelte:1099`, à l'intérieur de `Dialog.Root` `:811`). L'issue propose le
  `window.confirm` de `BankProfileList.svelte:30` / `RulesList.svelte:73` ou un dialogue.
- **Retenu** : `window.confirm(i18nMsg('contact-persons-confirm-delete', …, { name }))`, `name` = prénom et nom ; refus
  → rien n'est envoyé. Même patron que six fichiers du dépôt (sept appels, `git grep -n "confirm(" -- '*.svelte'`) ; pas de dialogue imbriqué dans un dialogue bits-ui (piège
  de focus et d'empilement pour un P3). Le texte dit qu'aucun écran ne la rétablit — vrai : la suppression archive
  (`active = FALSE`, `contact_persons.rs:158`, audit `contact_person.archived`) et aucune route ne désarchive.
- **Conséquence assumée** : un dialogue natif n'apparaît pas dans une capture Playwright ; la recette visuelle de ce
  correctif se fait sans capture, et le rapport le déclare.
- **Écartée** : un `Dialog` imbriqué (captures possibles, mais nouveau composant et focus imbriqué).
- **Réversible** : oui.

### C-29-9-11 — Ordre : en dernier de la série de la réconciliation, dans la v0.13.4

- **Contexte** : #649 touche `ReconciliationProposals.svelte`, que modifient la 29-3 (paragraphe de priorité avant la
  table), la 29-6 (case, *Accepter* grisé, indication dans la rangée des boutons) et la 29-8 (troisième bouton de
  ligne, modale), et le passage du manuel `user-manual.tex:2045-2057` que les trois réécrivent.
- **Retenu** : la 29-9 se rebase après elles et se merge en dernier ; son ajout à l'écran se place **après la table**
  (`</table>`, `:401`), zone qu'aucune des trois ne touche. Elle part dans la v0.13.4.
- **Réversible** : oui.
- *Annotation du 2026-10-14 (validation P2, F8)* : « zone qu'aucune des trois ne touche » n'est pas établi pour la
  29-8, qui ajoute une `LinkSettlementModal` à ce composant sans en prescrire l'emplacement (fiche 29-8 `:370`) — son
  emplacement naturel est après `</table>`, à côté des deux autres modales. Règle retenue (AC1) : au rebase, l'avis et
  le bouton se placent **immédiatement** après `</table>`, avant toute instance de modale, la sienne comprise.

### C-29-9-12 — Tests de bout en bout : deux specs neuves, aucun pour #655

- **Contexte** : #649 et #646 changent un parcours ; #655 ne change aucun parcours, et la course qu'il corrige n'est pas
  reproductible de façon déterministe à travers un vrai serveur.
- **Retenu** : `reconciliation-show-older.spec.ts` (import CAMT synthétique de 101 transactions, bouton, 101e visible)
  — *corrigé le 2026-10-14 (validation P1, R5)* : la fiche dit, et c'est ce qui vaut, **501** transactions (paliers 100
  puis 500, avis plafonné), la **150e** absente avant le clic et présente après ; entrées uniques (F11) ;
  et `contact-persons-delete.spec.ts` (refus → la personne reste ; acceptation → elle part, et le serveur ne la rend
  plus). #655 est couvert par Vitest, promesses résolues dans l'ordre inverse.
- **Réversible** : oui.

### C-29-9-13 — #655, second cas : l'effet du profil dépend d'un `$derived` de l'identifiant (validation P1, F4)

- **Contexte** : `page.params` est un `$state.raw` réassigné par `Object.assign(page, …)` à chaque navigation où
  `page_changed` est vrai — URL, erreur, formulaire ou données changés (`@sveltejs/kit` 2.55.0, `state.svelte.js:37`,
  `client.js:763-772`, `:1888` ; sinon `clone_page(page)`, `:707`, garde la référence — précisé en validation P2, F5) ; un effet qui lit `page.params.id` se rejoue donc à
  `id` égal. `BankProfileForm` initialise ses champs **une fois** depuis `existing` (`BankProfileForm.svelte:46-60`) :
  à id changé, il doit être remonté ; à id égal, il ne doit **jamais** l'être (saisie en cours). Décision de
  l'orchestrateur : le geste le plus étroit, sans démontage ni perte de saisie au même id.
- **Retenu** : `const rawId = $derived(page.params.id)` ; l'effet ne lit que `rawId`. Un `$derived` compare par `===`
  (`svelte/src/internal/client/reactivity/equality.js:4-6`, Svelte 5.55.1) : à id égal, il garde sa valeur et l'effet
  ne se rejoue pas. La **chaîne** est dérivée, non le nombre (`NaN !== NaN` relancerait l'effet à chaque
  réassignation de `page.params` sur un id invalide). En tête de l'effet — donc d'un chargement réel — `++loadSeq` (avant la branche « ID
  invalide »), `errorMessage = null`, `isLoading = true` ; garde de séquence sur `profile`, `errorMessage`,
  `isLoading`. `isLoading = true` est **gardé** : établi au code qu'il ne s'exécute qu'à id changé, où il est
  nécessaire. Tests V13 (id égal : un seul appel, saisie intacte), V14, V15, V16 ; fixture qui **réassigne** `params`.
- **Écartées** : `{#key rawId}` autour du formulaire (démonte aussi, mais c'est l'effet qu'il faut borner, et deux
  mécanismes pour un) ; `untrack(() => page.params)` (n'exprime pas « l'id seul ») ; ne pas poser `isLoading = true`
  (le profil 6 s'afficherait avec les champs du 5).
- **Réversible** : oui.

### C-29-9-14 — #655, premier cas : `loading` du détail d'import n'est pas gardé (validation P1, F7 et R8)

- **Contexte** : la fiche initiale gardait `loading` dans le `finally`. Or `load()` ne remet jamais `loading` à
  `true`, et les deux déclencheurs de relecture ne sont rendus que dans `{:else if detail}` (`[id]/+page.svelte:96`),
  donc après la première lecture : un `finally` périmé écrit `false` sur `false`.
- **Retenu** : garde de séquence sur `detail` et `errorMessage` seulement ; `loading = false` reste nu dans le
  `finally`, avec un commentaire qui dit pourquoi et qu'il faudra le garder le jour où `load()` le remettra à `true`.
- **Écartée** : garder `loading` (garde intuable : aucun test ne peut la rougir) ; ajouter `loading = true` à `load()`
  pour la rendre observable (démonterait les lignes à chaque relecture — un changement d'écran hors de #655).
- **Réversible** : oui.

### C-29-9-15 — #649 : le saut de défilement au clic est déclaré, non corrigé (validation P1, F9)

- **Contexte** : `{#if loading}` (`ReconciliationProposals.svelte:266-269`) remplace la liste par le message de
  chargement pendant tout rechargement ; au clic sur *Afficher aussi les plus anciennes*, en bas de page, la page
  remonte.
- **Retenu** : déclarer (AC2, Dev Agent Record) ; `load()` inchangé. Le même saut existe déjà après *Accepter* et
  *Rejeter*.
- **Écartée** : ne passer `loading` à `true` que si la liste est vide — la liste resterait cliquable pendant un
  rechargement pour **tous** les déclencheurs (dont un lot en vol), un changement de comportement hors du P3.
- **Réversible** : oui.

### C-29-9-16 — #649 : le commentaire de module de `kesh-db` n'est pas réécrit (validation P1, R3/F3)

- **Contexte** : `crates/kesh-db/src/repositories/reconciliation.rs:18-19` dit « paginées (limit hardcodé 100 au
  caller, cf. L24) », trouvé par le grep élargi.
- **Retenu** : laissé tel quel, cité au Dev Agent Record : le défaut de 100 est bien fixé par la route appelante, et
  la story ne change aucun code de production Rust — le commentaire reste vrai. Le préciser ferait entrer un
  repository `kesh-db` dans le diff pour un commentaire, et avec lui l'exception *kesh-db* (gate complet à chaque
  patch de la boucle de revue).
- **Écartée** : le réécrire (« défaut 100, plafond 500, fixés par la route »).
- **Réversible** : oui.
- *Annotation du 2026-10-14 (validation P2, R1 = F1) — **décision de l'orchestrateur : le « Retenu » ci-dessus est
  renversé**, conservé pour la trace.* Les deux raisons étaient fausses. (1) « Le commentaire reste vrai » : il est
  faux **depuis la Story 8-4** (`22fd6d4a`, le commit qui l'a écrit) — l'appelant ne code pas 100 en dur, la route
  prend `?limit=` de 1 à 500, 100 par défaut (`routes/reconciliation.rs:598-602`), et passe `limit + 1` (`:622-626`) ;
  après la story, l'écran demande lui-même 500. Un site du symptôme de #649 était donc trié sur un motif faux.
  (2) « L'exception *kesh-db* : gate complet à chaque patch de la boucle » : surestimée — CLAUDE.md dit « dès qu'un
  **patch** touche … un repository » ; elle vaut pour **le commit qui touche ce fichier**, celui de T1, et pour un
  patch de revue qui le retoucherait, pas pour toute la boucle. **Retenu désormais** : le commentaire est **réécrit en
  T1** (« `limit` fourni par la route : défaut 100, plafond 500, sentinelle `limit + 1` pour `hasMore` »), le renvoi
  `L24` disparaît ; gate complet au commit de T1 ; aucun code de production Rust ne change. **Écartée** désormais : le
  laisser comme angle mort assumé (un commentaire faux qu'on sait faux, pour éviter un gate).

### C-29-9-17 — #655, second cas : `onUpdated` gardé, sur l'identifiant courant et non sur une fermeture (validation P2, F2)

- **Contexte** : `bank-import/profiles/[id]/+page.svelte:33-35`, `onUpdated(updated)` écrit `profile` sans condition.
  Une sauvegarde du profil 5 qui se termine après une navigation vers 6 (et après la lecture de 6) écrit `profile = p5` ;
  le formulaire monté garde les champs de 6 (initialisés une fois, `BankProfileForm.svelte:46-60`) mais `existing` = p5,
  et la sauvegarde suivante écrit les valeurs de 6 dans le profil 5. Latent, comme le reste du second cas (aucun lien
  d'un profil à l'autre au sol). **Décision de l'orchestrateur** : `onUpdated` reçoit la même garde de séquence que le
  chargement — une sauvegarde qui finit après une navigation vers un autre profil n'écrit plus `profile`.
- **Retenu** : la garde se pose **dans** `onUpdated`, évaluée à l'appel : `if (updated.id !== Number(rawId)) return;`
  puis `profile = updated`. C'est la forme que prend la décision de l'orchestrateur pour une écriture que la page ne
  lance pas elle-même : la page ignore toute écriture qui ne relève plus du profil affiché, comme `mySeq === loadSeq`
  ignore une lecture périmée. Test V17, mutation M19.
- **Écartées** : un numéro de séquence capturé dans une fermeture passée au formulaire (`onSuccess={onUpdatedFor(seq)}`)
  — **ne tient pas** : Svelte 5.55.1 compile `let { existing, onSuccess } = $props()` suivi d'un appel après `await` en
  `$$props.onSuccess?.($$props.existing)` (compilation d'un composant d'essai, 2026-10-14, hors dépôt), si bien que le
  formulaire démonté de 5 appellerait le gestionnaire **courant**, celui de 6, et que la garde passerait ; modifier
  `BankProfileForm` pour qu'il capture l'identifiant au départ de la sauvegarde (hors des deux sites de #655) ; sortir
  le site du périmètre (C-29-9-9) — l'orchestrateur a tranché pour la garde.
- **Conséquence assumée** : retour de 6 à 5 avant la fin de la sauvegarde de 5 → la réponse est acceptée ; c'est juste
  (même profil, valeurs enregistrées), et une garde strictement séquentielle l'aurait refusée sans gain.
- **Réversible** : oui.

### C-29-9-18 — #655, second cas : la dépendance synchrone de `getBankProfile` s'écrit au site, sans `untrack` (validation P2, F6)

- **Contexte** : « l'effet ne dépend que de l'identifiant » (C-29-9-13) suppose aussi que la partie synchrone de
  `getBankProfile(id)` ne lit aucun signal. Vrai au sol (`request` → `buildHeaders` → `fetchWithRetry` →
  `fetchWithTimeout`, `api-client.ts:181-199`, `:267-280`, `:329-336`, `:392-399`) ; V13 mocke l'API et ne le verrait
  pas changer.
- **Retenu** : un commentaire au site qui écrit cette dépendance et nomme le remède (`untrack(() => getBankProfile(id))`)
  pour le jour où une lecture réactive entrerait dans `request`.
- **Écartée** : `untrack` dans le code dès maintenant — garde qu'aucun test ne peut rougir au sol (même règle que
  C-29-9-14 pour `loading`).
- **Réversible** : oui.

### C-29-9-19 — #678 entre dans la story ; le titre dit « quatre », la clé reste (amendement du 2026-10-14)

- **Contexte** : décision de l'orchestrateur, après la clôture de la validation (P3 ciblée) : #678 (`bug`, `P4`) touche
  le même composant que #649, et la règle de release de la v0.13.4 (moins d'issues ouvertes qu'au tag v0.13.3)
  l'exige. La clé `29-9-trois-petits-correctifs` nomme le fichier de la fiche, la clé de `sprint-status.yaml`, le
  registre et trois prompts de validation.
- **Retenu** : AC11 et T10 neufs ; AC1-AC10 inchangés sur le fond, renvois datés seulement. Titre de la fiche
  « Quatre petits correctifs … des cases sans nom » ; **clé inchangée**, ni fichier renommé, ni clé de sprint changée.
  Comme pour les trois autres, un #678 fautif sort du périmètre plutôt que de retenir les autres (C-29-9-1).
- **Écartées** : renommer la clé et le fichier (casserait les renvois des prompts P1-P3, des rapports de validation et
  du registre, pour un mot) ; une story séparée pour #678 (même composant que #649, même spec E2E, rebase sur la même
  29-6 : deux branches se marcheraient dessus).
- **Réversible** : oui.

### C-29-9-20 — #678, libellé de la case : deux clés à variables, les valeurs telles que la ligne les affiche

- **Contexte** : #678 demande un libellé qui nomme la transaction (date, montant, contrepartie). La contrepartie est
  `string | null` (`reconciliation.types.ts:18`) ; la cellule affiche `?? ''`.
- **Retenu** : `aria-label` sur la case, deux clés — `reconciliation-labels-select-transaction` (avec `{ $counterparty }`)
  et `…-no-counterparty` —, choisies par la véracité de `counterpartyName` ; variables `{ $date }`, `{ $amount }`,
  `{ $currency }` remplies des **chaînes** de l'API, sans reformatage : le libellé dit ce que la ligne montre. L'en-tête
  vide de la colonne ne change pas.
- **Écartées** : une seule clé avec une contrepartie vide (libellé terminé par « , ») ; un sélecteur Fluent sur une
  valeur sentinelle (`[none]`), qu'aucune clé voisine n'emploie ; `aria-labelledby` vers les cellules de la ligne
  (exigerait des `id` par cellule, et lirait aussi la colonne *Candidate*, que #678 ne demande pas) ; formater date et
  montant (le libellé dirait autre chose que la ligne) ; un nom de colonne « Sélection » dans l'en-tête (n'identifie
  pas la transaction).
- **Réversible** : oui.

### C-29-9-21 — #678, annonce : une région qui englobe le `{#if}` de la 29-6 ; un test de la 29-6 amendé en delta

- **Contexte** : C-29-6-25 l'a écrit : une annonce fiable demande une région présente **avant** le changement. Le
  `data-testid` de l'indication est éprouvé par présence et absence (Vitest, E2E de la 29-6) et par position (test
  C-29-6-18 : parent = la rangée, dernier enfant = l'indication).
- **Retenu** : `<div role="status" aria-live="polite" aria-atomic="true" data-testid="reconciliation-selection-status">`
  à la place du bloc `{#if}`, l'englobant ; le `<p>` de la 29-6 à l'octet, dedans. Contrat de présence et d'absence
  intact, E2E de la 29-6 inchangée ; **seul** le test de position change, en delta (ses deux assertions de position
  suivent l'imbrication ; intention et volet repli à l'octet). V21 prouve que le nœud de la région est le même avant et
  après le changement (M27).
- **Écartées** : un `<p>` toujours rendu, vide hors indication (casse `queryByTestId → null` et `toHaveCount(0)` : le
  contrat que l'orchestrateur demande de garder) ; un `data-testid` conditionnel sur un `<p>` toujours rendu (garde
  les tests à la lettre mais fait d'un sélecteur de test un indicateur d'état — surprenant pour qui le relira) ;
  `role="status"` sur la rangée entière (les libellés des boutons entreraient dans l'annonce) ; `role="status"` sur
  le `<p>` inséré (écarté par C-29-6-25 : pas d'annonce fiable).
- **Réversible** : oui.

### C-29-9-22 — #678 : ni manuel, ni capture neuve ; un volet E2E dans la spec de #649 ; périmètre borné à l'écran

- **Contexte** : aucun texte visible ne change ; la spec `reconciliation-show-older.spec.ts` de #649 importe 501
  transactions sans facture, donc toutes sans proposition et cochables (29-6).
- **Retenu** : manuel inchangé (le PDF régénéré par T6 se contrôle tout de même sur la phrase de la 29-6) ; une puce
  de CHANGELOG ; volet E2E dans la spec de #649, avant son clic (nom accessible = gabarit fr-CH rempli des textes des
  cellules ; région présente, puis indication dedans, puis absente) ; témoin visuel : les captures de la spec de la
  29-6, relancée sans modification. Seules la case de cette table et l'indication de la 29-6 sont revendiquées ; les
  23 autres cases du frontend (24 sites à `fcd2ae75`), une case « tout cocher » et les autres messages de l'écran
  restent hors périmètre.
- **Écartées** : une phrase d'accessibilité au manuel (le manuel ne décrit nulle part les noms accessibles) ; une spec
  E2E neuve pour #678 (un import de plus pour ce que la spec de #649 offre déjà) ; modifier la spec de la 29-6.
- **Réversible** : oui.

### C-29-9-23 — #678 dépend de la 29-6 mergée ; T10 se place entre T4 et T5

- **Contexte** : l'indication et la case sur toute ligne sont des ajouts de la 29-6, non mergée le 2026-10-14 ; T5
  recompte `sitesTotal`, que T10 change.
- **Retenu** : T10, numéroté à la suite pour ne renuméroter aucune tâche validée, s'exécute après T4 et avant T5 ; il
  attend la 29-6 sur `main` (l'ordre C-29-9-11 l'imposait déjà à toute la story). Numéros de ligne de l'AC11 relus en T0.
- **Écartées** : renuméroter T5-T9 (casserait les renvois des rapports de validation) ; écrire l'AC11 sur le sol de
  `main` (l'indication n'y existe pas).
- **Réversible** : oui.

### C-29-9-24 — #678 : deux appels littéraux, un test de relevé, les cinq clés de la 23-5 intactes

- **Contexte** : les trois gardes i18n ne retiennent que les appels à clé littérale (`site.arg?.kind !== 'literal'`) ; une clé choisie par un ternaire à l'intérieur de l'appel leur serait invisible (P4-3).
- **Retenu** : deux appels `i18nMsg` à clé littérale sous un ternaire sur `counterpartyName` ; un test neuf dans `i18n-entrees-a-variables.test.ts` qui nomme les deux clés (mutation M30) ; la borne des cinq clés de la 23-5 n'est pas modifiée.
- **Écartées** : une clé calculée (invisible aux gardes) ; élargir la borne des cinq clés (elle porte le titre « cinq ») ; ne rien ajouter et le déclarer au Dev Agent Record (la garde resterait muette).
- **Réversible** : oui.

### C-29-9-25 — #678 : « repli = valeur fr-CH » tenu par V19 et V20, non par la garde générique (P5-1)

- **Contexte** : `i18n-repli-divergent-actif.test.ts:146` ne retient que les clés à deux replis divergents ou inscrites à `REPLIS_A_SITE_UNIQUE` (`:216-233`) ; les deux clés de l'AC11, à site unique, n'y sont pas. La fiche affirmait le contraire (AC11, AC8).
- **Retenu** : retirer l'affirmation ; V19 et V20 lisent leur attendu dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` (précédents `UnrejectTransactionAction.test.ts:31`, `ReconciliationProposals.test.ts:672`) — le mock i18n rend le repli, un repli divergent rougit donc ces tests. Pour les clés de l'AC8 (`more-hidden`, `more-capped`), l'égalité repli/catalogue n'est gardée par aucun test : angle mort assumé, écrit à l'AC8.
- **Écartées** : inscrire les clés à `REPLIS_A_SITE_UNIQUE` (liste fermée de la 23-x, hors périmètre, plus coûteux) ; un test d'égalité neuf pour l'AC8 (périmètre).
- **Réversible** : oui.

### C-29-9-26 — #649 : l'avis plafonné et le manuel nomment aussi le rattachement de la 29-8 (développement, 2026-10-15)

- **Contexte** : la fiche, écrite avant la 29-8, énumère trois gestes qui font sortir une transaction de la liste (*accepter, affecter manuellement, éclater*). Sur la base rebasée (`4b358373`), la 29-8 en ajoute un quatrième, *Rattacher à un règlement*, pour les crédits en CHF. Aucune règle métier ne change : c'est l'énumération d'un texte.
- **Retenu** : `reconciliation-labels-more-capped` ×4 et le paragraphe neuf du manuel ajoutent « ou rattachées à un règlement » (de : « einer Zahlung zugeordnet », en : « linked to a payment », it : « collegate a un pagamento », termes du bouton de la 29-8). Le mot « rejet » reste absent (V4).
- **Écartées** : le texte de la fiche à la lettre (il dirait faux par omission dès la 29-8 mergée) ; une formule générique « traitées » (moins précise que la liste, qui écarte nommément le rejet).
- **Réversible** : oui.

### C-29-9-27 — #655 : la doublure de `page` est un objet à accesseurs, pas une classe (développement, 2026-10-15)

- **Contexte** : la fiche prescrit `class FakePage { params = $state.raw(…) }` ; or `i18n-libelle-en-dur.test.ts` (« la prémisse de la limite "méthodes de classe" tient encore — aucune classe dans src/ ») rougit dès qu'une classe entre dans `src/`, fixture de test comprise.
- **Retenu** : `let params = $state.raw(…)` au module et un objet `fakePage` à accesseur `get`/`set params` — même sémantique (la navigation réassigne un `$state.raw`), vérifiée par M15 rejouée sur la nouvelle fixture (tuée par V13).
- **Écartées** : garder la classe et exempter le fichier dans la garde (élargir une garde d'une autre story pour une fixture) ; un `$state` profond (ne réassigne pas, M15 survivrait — F4 de la validation P1).
- **Réversible** : oui.

### C-29-9-28 — #646 : V11 affirme aussi que la question est posée (développement, 2026-10-15)

- **Contexte** : V11 (acceptation ⇒ suppression) décrit le comportement du sol ; écrit tel quel, il était vert avant le correctif, alors que T4 le veut rouge.
- **Retenu** : V11 vérifie aussi que `confirm` est appelé une fois ; rouge au sol pour cette raison, vert après ; M12 le rougit toujours.
- **Écartées** : déclarer V11 vert au sol par construction (comme V1, V12) — possible, mais il n'aurait alors rien prouvé de la story.
- **Réversible** : oui.

### C-29-9-29 — PDF : seul `user-manual.pdf` est versé (développement, 2026-10-15)

- **Contexte** : `make fr` régénère les trois PDF ; `admin-manual.pdf` et `marketing-brochure.pdf` changent d'octets sans que leurs `.tex` aient bougé.
- **Retenu** : les deux sont restaurés depuis HEAD ; seul le manuel utilisateur, dont le `.tex` change, est commité. Moins de PDF à régénérer au rebase sur la 29-8 et sur `main`.
- **Écartées** : verser les trois (bruit binaire, conflits de rebase sans contenu).
- **Réversible** : oui (`make fr`).

### C-29-9-30 — E2E de #649 : un seul import, quatre captures, et le « sombre » est clair (développement, 2026-10-15)

- **Contexte** : la fiche demande les captures clair et sombre des deux états. Importer 501 transactions par schéma doublerait le coût ; et Kesh n'a pas de thème sombre (C-28-1-7, écrit au site de `button.svelte`) : `emulateMedia({ colorScheme: 'dark' })` rend la page claire, comme les captures « sombre » de la 29-6.
- **Retenu** : un seul import ; chaque état capturé deux fois en basculant `emulateMedia`. Les captures « sombre » sont claires, et c'est déclaré ; la règle B2 (jetons du thème seulement) est tenue au code (`text-text-muted`, `border-text`).
- **Écartées** : deux scénarios à 501 transactions chacun ; ne capturer qu'en clair (la fiche demande les deux, et le jour d'un thème sombre la spec servira).
- **Réversible** : oui.

[#649]: https://github.com/guycorbaz/kesh/issues/649
[#655]: https://github.com/guycorbaz/kesh/issues/655
[#646]: https://github.com/guycorbaz/kesh/issues/646
[#678]: https://github.com/guycorbaz/kesh/issues/678

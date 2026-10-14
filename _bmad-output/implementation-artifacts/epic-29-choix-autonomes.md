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

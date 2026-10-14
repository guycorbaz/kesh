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
- **Réversible** : oui.

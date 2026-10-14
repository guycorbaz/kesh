# Epic 29 — registre des choix autonomes, Story 29-2

*Registre propre à la story 29-2 (#529), à verser par **union** dans `epic-29-choix-autonomes.md` par l'orchestrateur.
Fiche : `29-2-paiement-tardif.md`.*

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

### C-29-2-16 — Un `\label{sec:algorithme-matching}` neuf pour le renvoi du § des rappels

- **Contexte** : AC8 demande, à `:1305`, un renvoi « à `\S\ref{sec:facture-posterieure}` ou à l'algorithme » ; le
  § *Algorithme de matching* n'avait pas de label, et c'est lui qui dit la règle.
- **Retenu** : poser `\label{sec:algorithme-matching}` (libre au dépôt, `grep -rn algorithme-matching docs/manual`
  vide avant), et y renvoyer depuis `:1305` et la FAQ.
- **Écartée** : renvoyer à `sec:facture-posterieure` (il traite de la borne haute, pas de l'ancre).
- **Réversible** : oui.

[#529]: https://github.com/guycorbaz/kesh/issues/529
[#666]: https://github.com/guycorbaz/kesh/issues/666
[#668]: https://github.com/guycorbaz/kesh/issues/668

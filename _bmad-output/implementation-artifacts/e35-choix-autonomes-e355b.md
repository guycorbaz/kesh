# Epic 35 — registre des choix autonomes de la Story 35-5b (clé `e355b`)

Registre de l'agent de développement (Opus 5.5, worktree `kesh-e355b`), tenu par union
(`docs/agents-orchestration.md` § 1). Chaque choix : contexte, option retenue, autres options,
réversibilité. Option prudente et réversible par défaut.

## C-e355b-1 — Le module de tests de `kesh-db invoices.rs` passe entièrement en date fixe dès le squelette

- **Contexte** : D9 prescrit une date fixe pour les six sites du module (`fn today()` et cinq
  `CURDATE()`) ; la fiche prévoyait au squelette un compte de garde « 10 → 7 », c'est-à-dire des
  sites de tests encore à l'heure réelle après le squelette.
- **Retenu** : `fn today()` rend `2026-06-15` (dans l'exercice 2020-2030 de la base de dev semée),
  les cinq `CURDATE()` deviennent des liens sur `today()`, et chaque appel de
  `list_by_company_paginated` / `due_dates_summary` du module reçoit `today()` (ou `t`, la même
  date) — dès le squelette, car ces appels changent de signature à ce commit. Le compte du fichier
  tombe à 0 au squelette, non à 7.
- **Autres options** : garder `Utc::now()` au squelette puis convertir à l'implémentation (deux
  passages sur les mêmes lignes, sans gain de preuve : ces tests ne lisent aucune horloge).
- **Réversibilité** : totale (tests seuls).

## C-e355b-2 — Paramètre `aujourd_hui` de `list_for_export` placé avant `max_rows`

- **Contexte** : la fiche ne fixe pas l'ordre des paramètres.
- **Retenu** : `list_for_export(pool, company_id, query, aujourd_hui, max_rows)` — la date suit la
  requête qu'elle borne, comme dans `list_by_company_paginated` et `due_dates_summary`. Les deux
  types diffèrent (`NaiveDate`, `i64`) : le compilateur refuse toute inversion.
- **Réversibilité** : totale.

## C-e355b-3 — La fiche d'une facture prend la date du service (`FicheFacture::aujourd_hui`)

- **Contexte** : le squelette faisait lire `state.horloge` à `get_invoice` ; la garde
  `services_motif::ac13_ix_les_seuls_usages_de_state_d_un_handler_migre` (Story 32-4b) n'admet,
  dans un handler migré, que `State(state)`, `state.lecture()`, `state.trace()` et `state.i18n`
  — rouge au gate du squelette (`gate-squelette-test-fast.log`, commit `777664745`).
- **Retenu** : la règle de D1 (« quand une route convertit ce qu'un service a rendu, elle emploie
  la date du service »), étendue à la fiche : `services::factures::detail` rend
  `FicheFacture { …, aujourd_hui }`, lu sur `lecture.aujourd_hui()` ; la route le passe à
  `from_parts`. La garde AC13 (ix) reste telle quelle.
- **Autres options** : ajouter `state.horloge` aux formes permises de la garde (élargit une garde
  d'une autre story pour un seul site) ; lire la date dans la route par `state.lecture()` (une
  seconde `Lecture` construite pour rien).
- **Réversibilité** : totale (un champ de type de service, sans effet sur le contrat HTTP).

## C-e355b-4 — Où vivent les tests neufs

- **Contexte** : les AC1 à AC4 exigent un serveur à horloge arrêtée et le montage « exercices
  2025 et 2026 » ; la fiche ne nomme pas de fichier.
- **Retenu** : AC1 à AC4 dans `crates/kesh-api/tests/jour_suisse_nuit_e2e.rs` (35-5a), qui a déjà
  `spawn_app(pool, instant)` et `monter` — un fichier neuf aurait recopié ces cent lignes (DRY) ;
  AC5 dans `mcp_outils_e2e.rs` (banc MCP, `banc_avec_horloge`) ; AC6 dans `fiscal_years_e2e.rs`
  (`spawn_app_a`, comme le test du chemin B de la 35-5a) ; le jour du dernier rappel (AC3, côté
  dépôt) dans `kesh-db/tests/dunning_eligibility.rs` ; M7 en test unitaire de
  `routes/invoices.rs`.
- **Réversibilité** : totale (tests).

## C-e355b-5 — AC8 : le rouge de `services_rollout` sous un instant antérieur au 1er juin 2026 (remédiation P1, A2)

*Ajouté le 2026-11-14, remédiation de la revue P1 (finding A2), sur décision de l'orchestrateur.*

- **Contexte** : sous les deux instants de la fiche (`2025-12-31T23:30:00Z`, `2026-03-10T23:30:00Z`),
  `services_rollout::ac11_a_les_services_lisent_en_lecture_seule` rougit (`nuit-B1.log`, `nuit-B2.log`) :
  son montage `poser_livres` pose une écriture à date fixe `2026-06-01` (`services_rollout.rs:206`) puis
  lit les postes ouverts à l'`asOf` par défaut — le site 18, que **cette** story fait suivre l'horloge de
  test. Un instant antérieur au 1er juin 2026 laisse l'écriture hors de la lecture ; la sonde **de jour**
  au 11 mars 2026 (`nuit-B3.log`) rend le même rouge : c'est la classe (b) de la 35-5a (instant antérieur
  aux dates fixes du montage, C-e355a-7), non un mélange de deux jours. Sans effet à l'heure réelle,
  postérieure au 1er juin 2026, qui ne recule pas.
- **Retenu** : laisser le test tel quel et le déclarer — écart à la lettre d'AC8 (« verts sous les deux
  instants »). Le fichier est **touché par la 35-5b** (cinq hunks de conversion, `factures::lister` →
  `.page`) ; ce n'est donc pas « un fichier hors 35-5b », et la fiche 35-5c ne le nomme pas : le cas est
  à tracer en commentaire de #1125 (tests à dates fixes de 2026), par l'orchestrateur — à noter qu'il ne
  rougira **jamais** à l'heure réelle, contrairement aux cas de #1125, qui rougiront dès 2027.
- **Autres options** : passer un `asOf` explicite au montage (`Some("2026-06-30")`) — une ligne dans un
  fichier que la story touche déjà, mais qui retire au test sa lecture « à l'`asOf` par défaut » ;
  avancer la date fixe du montage avant tout instant de campagne (modifie un test de la 32-4b sans
  rapport avec la nuit).
- **Réversibilité** : totale (aucun code).

## C-e355b-6 — Le site 16 bis revient à `d.date()` : `paid_at` n'est pas un instant (remédiation P1, E1 = B1)

*Ajouté le 2026-11-14, remédiation de la revue P1 (findings E1 MEDIUM, B1 LOW), sur décision de l'orchestrateur.*

- **Contexte** : D4 et C-e350-25 classaient `paid_at` parmi les « jours d'un instant enregistré ». Lu au
  code : les trois écrivains de production posent la **date du règlement à minuit** —
  `invoice_settlements_write.rs:391-392` (règlement, `settled_on.and_hms_opt(0, 0, 0)`), `:745` (passage
  en perte), `routes/reconciliation.rs:2584-2585` (rapprochement, `paid_at_candidate.and_time(00:00)`).
  `jour_suisse(minuit UTC)` rend la même date : la conversion était l'identité sur toute donnée écrite,
  et le test d'AC2-paiement la prouvait sur une valeur (`23:30:00`) qu'aucun écrivain ne produit.
- **Retenu** (option prudente) : le CSV revient à `d.date()`, commentaire juste (« la date du règlement,
  posée à minuit ») ; le test devient un règlement réel par la route, de nuit, `settledOn` du 10 — la
  colonne porte `10.03.2026` et `paid_at` vaut `2026-03-10 00:00:00` ; le manuel ne range plus le
  paiement parmi les heures en UTC, le CHANGELOG n'annonce plus de changement de la date de paiement de
  l'export, la doc d'`Invoice::paid_at` dit ce qu'il est.
- **Autres options** : garder `jour_suisse(d.and_utc())`, identité sur toute donnée réelle, « par
  prudence » (une conversion qui ne convertit rien, un commentaire qui l'explique, un test qui ne peut
  rougir sur aucune donnée réelle).
- **Réversibilité** : totale (aucune donnée écrite ; la sortie du CSV est la même avant et après sur les
  valeurs que Kesh écrit).

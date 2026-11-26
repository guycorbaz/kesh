# Story 46-7 : solde de la dette de l'epic

Status: done

*Esquisse du 2026-11-18 (renumérotée 46-7 le 2026-11-19 : la balance prend le 46-6). Revue du responsable de l'epic : approuvée. Story de solde (`CLAUDE.md` § *Priorités des défauts*, « Tarir la source, puis solder »).
Plafond : **2 issues**, fixé au fichier d'epic (§ *Stories*). **Dernière story de l'epic**, après 46-1 à 46-6b (et
46-5 si décidée), avant le tag de la release (0.21.0, prévision).*

## Amendement du 2026-11-22 (au développement) — la dette réelle de l'epic

La fiche date d'avant les stories 46-2 à 46-6b. Aucune issue de l'epic n'est encore **ouverte** sur GitHub : les
revues ont versé leur dette à la liste des issues à ouvrir de l'epic (`kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md`,
hors dépôt), où elle attend l'accord du responsable de l'epic. C'est donc cette liste, recoupée avec les Change Logs
des fiches 46-1 à 46-6b (AC-1), qui fait l'inventaire. La 46-5 est hors périmètre (décision en attente).

**Inventaire (dix points, ligne lue entre parenthèses)**

| # | Origine | Point | Label, priorité | Suite |
|---|---|---|---|---|
| D1 | 46-1 P1 (fiche 46-1, « Interaction avec la 47-3 », n° 20 de la liste) | un miroir de passif rangé sous un groupe d'actif dissout ce groupe à l'affichage groupé | enhancement ou documentation, P3 | **reste à ouvrir** : décision d'interface du responsable de l'epic, non un correctif |
| D2 | 46-2 P3 L2 (Change Log 46-2, « amélioration tracée ») | changer la vue du bilan pendant le chargement d'un autre rapport l'interrompt | enhancement, P4 | reste à ouvrir (enchaînement des chargements, risque de régression de l'écran) |
| D3 | 46-2 P3 L3 (Change Log 46-2, « L3 : noté, **non remédié** ») | la redemande du bilan efface l'erreur de l'autre onglet | enhancement, P4 | **n'était pas tracé** : versé à la liste par cette story (« on trace toujours ») ; reste à ouvrir |
| D4 | 46-3 P2 L3 (Change Log 46-3, « L3 (suffixe « (solde propre) » tronqué, production) ») | PDF groupé : la troncature d'une ligne « solde propre » coupe son suffixe | bug, P4 | **retenu, corrigé** |
| D5 | 46-3 P1 L2 (même entrée) | PDF groupé : limites en caractères, non en largeur réelle | bug, P4 | reste à ouvrir (toucherait les limites de tous les rapports groupés) |
| D6 | 46-3 P1 L4 (même entrée) | `texte_pdf::lignes` sans notion de page | technical-debt, P4 | reste à ouvrir |
| D7 | 46-3 P1 L5 (même entrée) | fidélité JSON/PDF de l'export groupé : un seul jeu de paramètres | technical-debt, P4 | reste à ouvrir |
| D8 | 46-4 P2 F2 (Change Log 46-4, « tracé dans `issues-a-ouvrir.md` ») | MCP : le test de fidélité saute les lignes de groupe | technical-debt, P4 | reste à ouvrir |
| D9 | 46-6b P2 LOW-3 (Change Log 46-6b, « LOW-3 … à ouvrir en issue ») | balance groupée en PDF : le premier compte d'un groupe peut passer seul en haut de page | bug, P4 | reste à ouvrir (plafond atteint) |
| D10 | revue d'intégration de l'Epic 46, L3 (liste, « Relevé par la revue d'intégration de l'Epic 46 (L3) ») | fermer puis rouvrir un groupe laisse « détail ajusté à la main » au PDF | bug, P4 | **retenu, corrigé** |

**Choix dans le plafond (2), AC-1.** Tous les points sont P4 sauf D1 (P3, mais une décision, non un correctif). Par
effet visible : les quatre `bug` (D4, D5, D9, D10). D10 met au PDF remis à une fiduciaire une mention fausse ; D4 rend
une ligne « solde propre » indiscernable d'une ligne de compte ; D9 n'est qu'une coupure de page, D5 demanderait de
recalculer les limites de tous les libellés groupés (risque de régression sur les trois rapports). Retenus : **D10 et
D4**, les plus visibles et les plus sûrs (un module chacun, AC-4).

**Hors plafond : LOW de revue jamais ouverts** (AC-4 : ils n'ont pas été ouverts ; ce sont des rédactions et des
commentaires, sans production) :
- revue P2 de la fusion de la 46-6b (`kesh-gate-logs/integration/epic-46-fusion-46-6b-p2.md`) : LOW-1 (« le titre et
  le sous-total d'un groupe répètent » : un seul des deux porte les montants dans un export donné), LOW-2 (redite et
  parenthèse mal placée de `docs/api-external.md`), LOW-3 (commentaires de l'E2E d'avant l'extraction de l'aide) —
  **corrigés** ;
- revue P4 d'intégration (`epic-46-revue-p4.md`), P4-1 et P4-2 : **déjà corrigés** par la reprise de la phrase à la
  fusion de la 46-6b (le texte actuel des quatre sites dit que titres et sous-totaux répètent les montants, et que
  l'export à plat porte lui aussi ses lignes de total, à filtrer par `Section`) ; constaté, rien à faire.

## Story

En tant que mainteneur, je veux solder, avant le tag, la dette ouverte par les stories de l'Epic 46, afin que l'epic
ne laisse pas de LOW sans suite.

## Critères d'acceptation (esquisse)

1. **AC-1** — Ne prend que des issues ouvertes par les stories 46-*, au plus le plafond fixé au kickoff. **La liste
   se tire des Change Logs et Dev Agent Records des fiches 46-1 à 46-6b** (et 46-5 si décidée), où chaque revue nomme
   les issues qu'elle ouvre (« on trace toujours ») ; chaque issue retenue y est citée avec la ligne lue. La mention
   « Epic 46, story 46-N » dans le corps d'une issue, à son ouverture, aide à la recherche mais n'est pas la source : rien
   ne l'impose aux autres fiches. Au-delà du plafond, le choix se fait par priorité (de `P1` à `P4`), puis par effet
   visible pour l'utilisateur ; chaque choix s'écrit au Dev Agent Record avec son motif.
2. **AC-2** — Chaque correctif suit la boucle de revue ; gates complets et E2E au dernier commit de code.
3. **AC-3** — Les issues 46-* au-delà du plafond restent ouvertes et sont nommées au bilan de l'epic (« on trace
   toujours ») ; aucune n'est fermée pour tenir le chiffre.
4. **AC-4** — Un correctif retenu qui toucherait plus de cinq modules n'est pas pris : il reste ouvert et se nomme
   (règle de splitting préventif). Les LOW des stories 46-1 à 46-6b que leur boucle de revue a corrigés sur place
   (§ *Tarir la source*) ne comptent pas dans le plafond : ils n'ont pas été ouverts.

## Tâches principales

- [x] Liste de la dette de l'epic ; choix dans le plafond (amendement du 2026-11-22)
- [x] D10 — `toggleGroup` rend au niveau un groupe ramené à son état de départ (`3247ef5ee`)
- [x] D4 — la ligne « solde propre » d'un PDF groupé garde son suffixe (`894040c9a`)
- [x] LOW de la revue de fusion de la 46-6b : phrase du CSV groupé, doc d'API, commentaires de l'E2E (`cffecfc3c`)
- [x] Liste des issues à ouvrir mise à jour (D4, D10 barrés ; D3 versé)

## Fichiers

- `frontend/src/lib/features/reports/reports-grouping.ts` (`openByLevel`, `toggleGroup` à quatre arguments, en-tête du
  module), `reports-grouping.test.ts`, `frontend/src/routes/(app)/reports/+page.svelte` (appel avec `row.depth`) ;
- `crates/kesh-report/src/pdf.rs` (`truncate_keeping_suffix`, `LibelleGroupe::de`, deux tests),
  `crates/kesh-api/tests/reports_export_e2e.rs` (`tronque_propre`, fidélité de la balance) ;
- `docs/manual/fr/user-manual.tex`, `docs/api-external.md` ; fiches 46-2, 46-3, 46-6b (phrase de leurs notes de
  version) ; commentaires de `frontend/tests/e2e/reports-balance-groupes.spec.ts` et `helpers/reports-groupes.ts`.

## Note de version

- **Fixed** — Rapports par groupes : fermer puis rouvrir un groupe (ou l'inverse) le rend à l'état que lui donne le
  niveau, et le PDF exporté ne dit plus « détail ajusté à la main » quand l'écran montre exactement le niveau choisi.
- **Fixed** — PDF des rapports par groupes : la ligne « (solde propre) » d'un compte de groupe à nom long garde son
  suffixe ; seul le nom est abrégé (sauf à partir de la profondeur 6 de la balance, dont la colonne est alors plus étroite que le
  suffixe).

## Dev Agent Record

**Prémisses vérifiées au code.**
- D10 : `toggleGroup` (`reports-grouping.ts`, `eb7498376`) ajoutait toujours le groupe à la liste opposée ; fermer
  puis rouvrir un groupe ouvert par le niveau laissait `open: [id]`, que l'API reprend dans son écho et que
  `grouped_level_line` (`pdf.rs`) imprime « , détail ajusté à la main » dès qu'une liste est non vide. Vrai.
- D4 : `LibelleGroupe::de` tronquait « <nom> (solde propre) » comme un tout par `truncate_with_ellipsis`. Vrai.

**Correctifs.**
- D10 : `openByLevel(detailLevel, depth)` reprend la règle d'`OptionsGroupes::est_ouvert` à listes vides (ouvert si
  `all` ou `depth < niveau` ; `depth` de la ligne = la `profondeur` passée à `est_ouvert`, `groupes.rs:837`) ;
  `toggleGroup` prend la profondeur de la ligne et n'inscrit le groupe dans une liste que si l'état voulu diffère de
  celui du niveau. Une entrée redondante d'une mémoire ancienne se ferme encore d'un clic (test). Aucun affichage n'en
  dépend : l'état montré reste celui que l'API rend. AC-3 et C-46-2-1 de la 46-2 amendés par une note datée du 2026-11-22 (revue P1, LOW-1) : un clic n'inscrit plus
  le groupe que si l'état voulu diffère de celui du niveau (citation d'origine rectifiée : l'AC-3 dit « ouvrir un groupe
  fermé par le niveau l'ajoute à `open` » sans restriction).
- D4 : `truncate_keeping_suffix(texte, suffixe, max)` abrège le nom seul ; le nom peut se réduire à « … » (numéro
  imprimé à part) ; plus étroit que le suffixe et son espace (balance au retrait maximal, 14 caractères), le tout est
  tronqué comme avant. Le test de fidélité de la balance suit, par `tronque_propre`, écrit indépendamment du rendu.
- Manuel (`.tex`, PDF non commité) et `docs/api-external.md` : les deux comportements ; phrase du CSV groupé reprise
  aux cinq sites (LOW-1), redite et parenthèse de la doc d'API (LOW-2), commentaires de l'E2E (LOW-3).

**Tests ajoutés** (recomptés, de `eb7498376` à `HEAD`) : `#[test]` de `pdf.rs` 80 → 82
(`truncate_keeping_suffix_garde_le_suffixe`, `ligne_solde_propre_a_nom_long_garde_son_suffixe`) ; `it(` de
`reports-grouping.test.ts` 25 → 29 (deux tests réécrits avec la profondeur, quatre neufs).

**Mutations** (hors dépôt, une à la fois, copie à cible cargo propre puis supprimée, témoin vert d'abord) :
- M2, `LibelleGroupe::de` rendu à la troncature du tout : **tuée**, `ligne_solde_propre_a_nom_long_garde_son_suffixe`
  panique sur son assertion (`pdf.rs:3409`, « profondeur 1 : Créances résultant de livraisons et prestations… ») ;
  `kesh-gate-logs/s46-7/mut-temoin.log`, `mut-m2.log`.
- M1, `toggleGroup` inscrivant toujours le groupe : **tuée**, trois tests en `AssertionError` (« fermer puis rouvrir
  un groupe ouvert par le niveau : listes vides », « ouvrir puis refermer… », « « Tous les comptes » : rouvrir… ») ;
  `mut-front-temoin.log` (29 passés), `mut-m1.log`.

**Captures** (`kesh-gate-logs/s46-7/captures/`) : `46-7-solde-propre-avant.pdf` et `-apres.pdf` (bilan d'essai, compte
de groupe à nom long, rendus `-1.png`) — avant : « Liquidités et placements à court terme cotés… », suffixe perdu ;
après : « Liquidités et placements à co… (solde propre) ». D10 ne change aucun écran (seule la requête et l'en-tête du
PDF) ; captures des écrans groupés reprises par les trois specs E2E au binaire de `cffecfc3c`.

**Gates.**
- Frontend, au dernier commit de code frontend (`cffecfc3c`, commentaires E2E compris ; rien de frontend depuis) :
  `npm run check` 0 erreur, 29 avertissements ; `lint-i18n-ownership` vert ; Vitest **4723 passés, 283 fichiers** ;
  `npm run build` vert (`front-*.log`).
- Backend, au dernier commit de code (`b2fbe46be`, mise en forme rustfmt), base remise à zéro : `scripts/test-fast.sh
  --ci` (`KESH_NO_MEMGUARD=1`, le bus systemd utilisateur refusant la connexion) : fmt et clippy workspace verts ;
  nextest **6260 exécutés, 6260 passés, 14 ignorés** (`gate-backend-final3.log`). Deux essais antérieurs :
  `final` (mem-guard sans bus, rien joué), `final2` (fmt rouge sur le test neuf, corrigé par `b2fbe46be`).
- E2E partiel (**3 specs**, celles qui lisent l'aide commune touchée par LOW-3, et qui exercent les clics d'accordéon
  et l'export) : `reports-balance-groupes`, `reports-groupes`, `reports-groupes-export` — **3 passés**, binaire et
  frontend de `cffecfc3c` (le commit suivant ne fait que remettre en forme un test Rust) ; `e2e.log`.
- E2E complet et `gate-ci.sh` : à la fin de l'epic, sur la branche d'intégration.
- Manuel : compilé dans une copie `git archive` hors dépôt (`make user`, rc 0, aucune ligne `!`), PDF supprimé ; texte
  aplati : les trois phrases neuves présentes (« suffixe » avec sa ligature ﬀ) ; `kesh-gate-logs/s46-7/manuel.txt`.

**Restent ouverts (AC-3, « on trace toujours »)** : D1, D2, D3 (versé par cette story), D5, D6, D7, D8, D9, à la liste
des issues à ouvrir de l'epic ; aucun retiré sans correctif.

## Change Log

- 2026-11-20 — Plafond fixé (2), position en dernier, AC-3 ; statut `drafted`.
- 2026-11-20 — validation P1 (Sonnet 5.5) : 0 C, 0 H, 0 M, 1 L, remédié. L : l'origine « ouverte par une story 46-* »
  n'avait aucune marque vérifiable, le choix au-delà du plafond aucun critère, le périmètre aucune borne : AC-1 et AC-4.
  Plafond 2 confronté au fichier d'epic (§ *Stories*) et à `CLAUDE.md` § *Priorités des défauts* : conforme. Axes
  exercés : fiche, fichier d'epic, `CLAUDE.md` ; non exercés : issues réelles (aucun accès à `gh`).
- 2026-11-20 — validation P2 (Claude Opus 5.5) : 0 C, 0 H, 1 M, 1 L, remédiés. M : la marque « Epic 46, story 46-N »
  posée par la P1 comme seule source de la liste n'est imposée par aucune autre fiche — AC-1 inapplicable → liste tirée
  des Change Logs et Dev Agent Records des fiches 46-1 à 46-6b, la mention restant une aide. L : « `P1` puis `P4` » →
  « de `P1` à `P4` ». Plafond (2) et borne de cinq modules confrontés au fichier d'epic : conformes. Rapport :
  `kesh-gate-logs/epic-46/validation/46-7-P2.md`.
- 2026-11-22 — revue P1 (Sonnet 5.5) : 0 C, 0 H, 0 M, 5 L ; boucle close (aucun finding au-dessus de LOW, remédiation sans
  code de production). LOW-1 : AC-3 et C-46-2-1 de la 46-2 amendés par note datée, citation de la 46-7 corrigée. LOW-2 :
  table partagée `crates/kesh-report/tests/fixtures/ouverture_par_niveau.json`, lue par un test Rust (`est_ouvert`) et un
  test Vitest (`openByLevel` via `toggleGroup`). LOW-4 : manuel et fiche disent « à partir de la profondeur 6 ». LOW-3 et
  LOW-5 : versés à la liste des issues à ouvrir de l'epic (LOW-5 : lisibilité assumée, sans correctif). Gates de clôture : fmt, clippy workspace, nextest `package(kesh-report) | binary(reports_export_e2e)` 321 passés (1 ignoré), Vitest des rapports 211 passés, `npm run check` vert ; PDF du manuel compilé dans une copie, phrase « à partir de la profondeur 6 » présente.
- 2026-11-22 — Développement (`bmad-dev-story`) : amendement daté (dix points de dette, plafond 2 : D10 et D4
  retenus), deux correctifs, LOW de la revue de fusion de la 46-6b corrigés, D3 versé à la liste ; gates ci-dessus ;
  statut `review`. Prompt de la revue P1 : `kesh-gate-logs/s46-7/revue-p1-prompt.md`.
- 2026-11-26 — **Suivis ouverts** (dette de la story, non réglée par la PR de l'epic) : #1643 (LOW-3 de la revue P1 : un groupe à sous-total nul disparaît après ouverture puis fermeture) ; #1644 (LOW-5 de la revue P1 : nom de la ligne « solde propre » abrégé aux profondeurs 2 et 3) ; #1645 (constat de la story : test Vitest de la table d'ouverture dépendant du répertoire courant).

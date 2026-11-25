# Story 55-1b : la présentation des rapports groupés dans l'URL

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D4. Issue : #À-VENIR (`bug`, P3, point b, comme
la 55-1a). Release : la prochaine planifiée après la 0.20.0. **Frontend seulement, aucune migration.** **Suit la 55-1a**
(le modèle d'URL, ses deux sens, ses clés) ; **base : la PR de l'Epic 46**, qui crée la présentation.

*Découpée de la 55-1 à sa validation P3 (signal de recyclage) : la 55-1a porte l'URL des rapports, la 55-1b y ajoute la
présentation de l'Epic 46 (hiérarchie, niveau, place des sous-totaux, groupes ouverts ou fermés à la main).*

## Story

En tant que comptable qui lit son bilan par groupes,
je veux que l'adresse d'un bilan, d'un compte de résultat ou d'une balance porte aussi sa présentation,
afin qu'un lien copié, un rechargement ou *Retour* me rendent le rapport tel que je le regardais.

## Modules touchés (règle de splitting préventif)

Trois : `frontend/src/lib/features/reports` (`reports-url.ts`, lecture de `reports-grouping.ts`),
`frontend/src/routes/(app)/reports` (page), `docs/manual/fr/user-manual.tex` ; plus `frontend/tests/e2e`.

## Ce qui se lit au code (`pr/46-rapports-groupes` `511e61d54`, par `git show`)

- `frontend/src/lib/features/reports/reports-grouping.ts` : `GroupingView { shown, detailLevel, subtotalPosition, open,
  closed }`, `defaultGroupingView`, `isDetailLevel` (entier 1 à `MAX_DETAIL_LEVEL = 20`, ou `all`), `readIdList` (entiers
  > 0, au plus `MAX_MANUAL_GROUPS = 500`, dédoublonnés), `sanitizeGroupingView` (objet **typé** ; deux listes qui partagent
  un identifiant sont ignorées toutes deux), `readGroupingView`, `writeGroupingView` (mémoire par société et rapport),
  `groupingParamsOf`, `sameGroupingParams`, `withEcho`.
- `frontend/src/routes/(app)/reports/+page.svelte` : `groupingViews` (les contrôles, par rapport), `displayedGrouping`
  (paramètres de la dernière réponse affichée, repris par l'export), `setGroupingView` (contrôles et mémoire),
  `changeGrouping` (redemande si un rapport est affiché ou en chargement, **rien sinon**), `displayedMatchesView`,
  `resyncActiveGroupedTab`, `afterGeneration`, `acceptGrouped` (reprend l'écho de l'API ; mémorise la vue **seulement**
  quand la requête portait des paramètres de regroupement — la vue `flat` n'en porte pas) ; `GroupingControls` rendu avant
  toute génération (le contrôle existe sans rapport affiché).
- Manuel de la PR 46, paragraphe de la présentation (`user-manual.tex:4240-4250` sur la PR 46) : « Le navigateur
  **retient votre présentation** … »

## Le modèle (D4), sur celui de la 55-1a

### Paramètres (bilan, compte de résultat, balance)

| Paramètre | Valeur valide |
|---|---|
| `view` | `grouped`, `flat` |
| `level` | `1`…`20`, `all` |
| `subtotals` | `header`, `footer` |
| `open`, `closed` | entiers > 0 séparés par des virgules, au plus 500 chacun, sans identifiant commun aux deux |

- **Lecture** : `reports-url.ts` convertit les chaînes (`level=3` → `3`, `open=1,2` → `[1, 2]`, refus de `open=1,a`) puis
  confie la validation à `isDetailLevel` et `sanitizeGroupingView` (aucune copie des règles, une conversion).
- **Écriture** : la vue **des contrôles** de l'onglet (`groupingViews[tab]`), toujours `view`, `level`, `subtotals`, et
  `open`, `closed` s'ils sont non vides, **en ordre croissant et sans doublon** ; ces paramètres suivent ceux de la 55-1a
  dans l'ordre canonique `view`, `level`, `subtotals`, `open`, `closed`.

### La page écrit

| Geste | URL |
|---|---|
| Contrôles de présentation, accordéons | **remplace**, clé enregistrée (la 55-1a ignore l'URL écrite par la page) : la 46-2 redemande comme aujourd'hui, ou ne demande rien sans rapport affiché ni chargement — l'URL écrite ne déclenche **jamais** de requête |
| Clic sur un onglet groupé | la clé du rapport affiché (55-1a) **avec la vue des contrôles** de l'onglet ; la règle de la 46-2 (`resyncActiveGroupedTab`) redemande si la vue affichée diffère, et l'écho remplace l'URL à la réponse |
| Réponse acceptée pour l'**onglet actif** | **remplace** avec la vue de l'écho (`withEcho` : les groupes disparus sortent des listes), clé enregistrée ; pour un autre onglet, rien |

### La page lit (`applyUrl` de la 55-1a, point 2, pour un onglet groupé)

- **Tout ou rien** : si l'URL porte au moins un paramètre de présentation **valide**, la vue est `sanitizeGroupingView` de
  ces seuls paramètres (les absents prennent la vue par défaut de la 46-2, non la mémoire), posée par `setGroupingView`
  (contrôles **et** mémoire, vue `flat` comprise) ; si elle n'en porte aucun de valide (`level=21` seul), la vue mémorisée
  (`readGroupingView`) s'applique et l'URL est **complétée** (remplace, clé enregistrée).
- **Rapport** (point 3 de la 55-1a) : la clé de l'onglet comprend la présentation — si la clé affichée porte les mêmes
  paramètres de rapport **et** `displayedGrouping` égale la vue (`sameGroupingParams`), rien ; mêmes paramètres de rapport,
  autre vue : redemande en gardant l'affichage (comme `changeGrouping`) ; sinon `generate()`.

## Critères d'acceptation

1. **AC-1 — Lien et rechargement.** Une URL d'un bilan groupé (niveau 2, un groupe ouvert à la main, sous-totaux en pied)
   ouverte dans un nouveau contexte qui reprend les cookies de la session, ou rechargée, affiche le bilan dans cette
   présentation, contrôles remplis, sans clic.
2. **AC-2 — L'URL prime sur la mémoire.** Avec une autre vue en mémoire pour ce rapport, l'URL l'emporte ; la mémoire prend
   ensuite la vue de l'URL (vue `flat` comprise).
3. **AC-3 — La mémoire complète l'URL.** `?tab=balance-sheet&fy=E` sans présentation : la vue mémorisée s'applique, l'URL est
   complétée par un `goto` qui remplace, sans nouvelle requête.
4. **AC-4 — Retour.** Niveau changé sur le Bilan (URL remplacée), clic sur un lien de compte, *Retour* : le Bilan dans la
   présentation choisie, sans requête.
5. **AC-5 — Règle de la 46-2 intacte.** Scénario X1 (niveau changé sur le Bilan, *Générer* sur le Compte de résultat, clic
   sur le Bilan) : le Bilan est redemandé dans la vue des contrôles à la fin de l'autre génération, et l'URL porte la vue
   des contrôles — rejoué dans le Vitest de la page avec un `page.url` qui suit `goto`.
6. **AC-6 — Aucune requête d'une écriture.** Un contrôle de présentation changé sans rapport affiché : URL remplacée,
   **zéro** requête.
7. **AC-7 — Écho.** Un groupe ouvert à la main puis disparu du plan : la réponse le retire des listes ; l'URL et la mémoire
   le retirent aussi ; un *Retour* vers cette entrée ne redemande rien.
8. **AC-8 — Valeurs invalides.** `level=0`, `level=21`, `open=1,a`, `open=5&closed=5`, `view=xyz` : ignorés, URL normalisée,
   aucune valeur invalide envoyée à l'API ; un paramètre invalide seul laisse la vue mémorisée.
9. **AC-9 — Export.** Les boutons PDF et CSV reprennent toujours la dernière réponse affichée (46-2, AC-6), inchangé.
10. **AC-10 — Manuel.** Paragraphe de la présentation de la PR 46 (`:4240-4250` sur la PR 46) : l'adresse porte la
    présentation, qui prime sur celle que retient le navigateur. `.tex` seul.

## Tests à écrire, et la preuve du rouge

- **Vitest** `reports-url.test.ts` (55-1a) : lecture et écriture des cinq paramètres, ordre et unicité des listes, valeurs
  invalides, tout ou rien.
- **Vitest de la page** (`reports-page-url.test.ts`, helper de la 55-1a) : AC-2, AC-3, AC-4, AC-5 (X1), AC-6, AC-7, AC-8.
- **E2E** `reports-navigation.spec.ts`, bloc 55-1b : AC-1 et AC-4 sur le seed CI (un plan groupé : le helper
  `tests/e2e/helpers/reports-groupes.ts` de la PR 46).
- Rouges sans le correctif (journal `kesh-gate-logs/s55-1b/`) ; mutations nommées : mémoire appliquée malgré l'URL ; vue
  `flat` d'un lien non mémorisée ; écho non reporté dans l'URL ; URL écrite par un contrôle réappliquée (requête).

## Recette visuelle

Captures (`kesh-gate-logs/s55-1b/`) : bilan groupé avant le lien ; après *Retour* ; lien collé dans un nouveau contexte.

## Tâches

- [ ] **T0** — Base `epic/55-navigation` avec la 55-1a.
- [ ] **T1** — Tests rouges.
- [ ] **T2** — Paramètres de présentation dans `reports-url.ts` ; écriture aux gestes et à l'écho ; lecture tout ou rien.
- [ ] **T3** — Manuel, E2E, captures ; gates.

## Gates

`cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` ; E2E des Rapports ;
suite complète selon le § *Gates* du fichier d'epic (Q6 : tant qu'elle n'est pas tranchée, E2E complet au dernier commit
de code de la story).

## Note de version

*Rapports* : l'adresse d'un bilan, d'un compte de résultat ou d'une balance porte aussi sa présentation (hiérarchie,
niveau, sous-totaux, groupes ouverts) ; elle prime sur celle que retient le navigateur. (refs #À-VENIR)

## Dev Notes

### Choix et écarts

- **C-55-1b-1** — Tout ou rien entre l'URL et la mémoire : mêler un niveau de l'URL et des listes de la mémoire donnerait
  une vue que personne n'a affichée (les listes dépendent du niveau).
- **C-55-1b-2** — L'URL porte la vue des **contrôles**, non celle du rapport affiché : c'est ce que la 46-2 fera afficher
  (redemande à la fin d'une génération en vol), et l'écho corrige l'URL à la réponse.

### References

- Fiche 55-1a ; fiche 46-2, AC-3, AC-5, AC-6, AC-10.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### Issues à ouvrir

(Points de revue non corrigés : sévérité, site, texte proposé ; source de la story de solde 55-4.)

### File List

## Change Log

- 2026-11-25 — Fiche écrite au découpage de la 55-1 (validation P3), reprend D4 et les findings V1-6, V1-7, V2-11 et V3-1,
  V3-6, V3-8, V3-12 de la 55-1 (`drafted`).

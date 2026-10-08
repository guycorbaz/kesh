# Epic 15 — registre des choix faits en autonomie

Le 2026-10-08, Guy a confié l'Epic 15 en développement **entièrement autonome** : aucune
question en cours de route, les choix tranchés par l'orchestrateur et **présentés ici, pour
sa revue en fin d'epic**.

Chaque entrée est écrite **au moment où le choix est fait**, pas reconstituée à la fin. Elle
dit le contexte, l'option retenue, les options écartées, et ce que coûterait de revenir
dessus.

**Autorisations reçues le 2026-10-08** : pousser, ouvrir et merger (squash) les PR une fois
la CI verte ; publier la release v0.13.0 ; conduire seul la revue de projet après
sauvegarde ; traiter dans l'epic les constats de recette P1/P2 de la v0.12.1.

**Décompte des bugs au kickoff** (§ *Priorités des défauts* du `CLAUDE.md`) : **49**, dont
14 P1 (9 de la TVA, en report assumé), 11 P2, 16 P3, 8 P4. Commande :
`gh issue list --state open --limit 500 --json labels --jq '[.[]|select([.labels[].name]|any(.=="bug" or .=="known-failure"))]|length'`

---

## C1 — Périmètre de l'epic

- **Retenu** : le jalon GitHub « Épique 15 » tel qu'il est (#235, #458, #459, #460, #461),
  **plus** les cinq P1 hors TVA (#427, #429, #434, #473, #474), **plus** le lettrage, qui
  n'avait pas d'issue et en reçoit une. L'epic se clôt quand le jalon est à zéro.
- **Sortent de l'epic** : 15-3 (versioning des parseurs, modèles, manuels embarqués) et
  15-4 (journaux personnalisables). Aucune issue ne les porte, aucune n'est un défaut, et
  l'epic livre déjà deux fonctionnalités (lettrage, justificatifs). Elles retournent au
  backlog, sans epic.
- **Écarté** : tout le périmètre d'origine de `epics.md`. L'epic aurait compté douze
  stories, et l'engagement « le nombre de bugs baisse » passe avant.
- **Réversible** : oui, en remettant les deux stories dans le registre.

## C2 — Ordre des stories

Les trois corrections P1 d'abord (15-5, 15-6, 15-7) : elles sont courtes, indépendantes,
et elles font baisser le décompte dès le début. Puis le lettrage (15-1a → b → c), les
justificatifs (15-2), et enfin les quatre issues du jalon sur les factures fournisseurs et
l'import (#458–#461).

| Story | Objet | Issues |
|---|---|---|
| 15-5 | Gardes de postabilité côté serveur : réconciliation, règles, réglages de facturation | #427, #429 |
| 15-6 | La créance juste : l'avoir crédite le compte de la vente ; un règlement ne peut viser le compte débiteurs | #473, #474 |
| 15-7 | La piste de contrôle de l'onboarding | #434 |
| 15-1a/b/c | Lettrage — socle, vue « ce qui reste ouvert », proposition et écran | (nouvelle issue) |
| 15-2 | Pièces justificatives sur les écritures | #235 |
| 15-8 | Import automatique du dossier, et le manuel qui le promettait | #459, #458 |
| 15-9 | Cycle de paiement des factures fournisseurs ; état « envoyée » des factures clients | #460, #461 |

*(La numérotation 15-8 et 15-9 peut changer à leur spécification ; ce registre le dira.)*

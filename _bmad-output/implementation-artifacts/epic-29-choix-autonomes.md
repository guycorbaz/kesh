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

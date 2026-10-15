# Epic 30 — registre des choix autonomes

Chaque choix tranché en autonomie : contexte, option retenue, alternatives écartées, réversibilité. Numérotation
`C-<story>-<k>`. Mise à jour **par union** (`docs/agents-orchestration.md` § 1) : chaque branche ajoute ses entrées dans
un registre propre à sa story (`epic-30-choix-autonomes-<clé>.md`), que l'orchestrateur verse ici au merge.

---

## Story 30-0 — Kickoff (doc seulement)

### C-30-0-1 — Une seule release, v0.14.0, en fin d'epic

- **Contexte** : Guy ne testera qu'en fin d'epic (« ces fonctions sont indispensables à la mise en production ») ; la
  29-1b, mergée, attend déjà la v0.14.0.
- **Retenu** : une seule release **v0.14.0**, après le merge de la dernière sous-story de #685, avec la 29-1b.
- **Écartée** : publier la 30-1 seule en v0.13.5 — sans #685, le processus voulu (saisir la facture depuis la pièce
  puis la régler par la réconciliation) reste à moitié praticable, et Guy n'en fera pas la recette.
- **Réversible** : oui.

### C-30-0-2 — #671 reste hors du périmètre, sauf part mécanique

- **Contexte** : #671 (P2, bug) demande de rattacher aux règlements déjà saisis les transactions comptabilisées par
  d'autres chemins — fiche fournisseur, écriture manuelle, paiement unique de plusieurs factures, lots pain.001. Sa
  première puce est le symétrique fournisseur de la 29-8, voisin de la 30-1.
- **Retenu** : la fiche de la 30-1 dit si ce symétrique sort mécaniquement de son travail ; si oui, elle l'intègre
  et l'issue reçoit un commentaire (elle ne se ferme pas tant que ses autres chemins restent). Sinon, rien.
- **Écartée** : faire de #671 une story de l'epic — quatre chemins hétérogènes, dont un (lots pain.001) « à
  vérifier » ; l'epic deviendrait le plus large de l'année pour une issue qui ne bloque aucune écriture.
- **Réversible** : oui.

### C-30-0-3 — La 30-1 avant la 30-2, fiches en parallèle

- **Contexte** : ordre proposé à Guy et non contesté ; les deux stories touchent les factures fournisseur, mais par
  des côtés différents (réconciliation contre import et saisie).
- **Retenu** : les deux fiches s'écrivent en parallèle, chacune dans son worktree ; le développement de la 30-1 passe
  en premier, celui de la 30-2 suit et se reporte sur `main` après le merge de la 30-1.
- **Réversible** : oui.

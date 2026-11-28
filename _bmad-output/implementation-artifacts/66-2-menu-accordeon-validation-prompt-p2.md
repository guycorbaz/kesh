# Validation P2 de la fiche 66-2 (menu latéral en accordéon, #852, #959) — prompt

Tu fais la **passe P2 de validation** de la fiche `_bmad-output/implementation-artifacts/66-2-menu-accordeon.md`, dans le
worktree `/home/gcorbaz/devel/kesh-66-2` (workflow `bmad-create-story`, mode *validate*). Contexte frais. Lis `CLAUDE.md`,
`docs/agents-orchestration.md` § 6, les issues #852, #959 et #247 (`gh issue view N --comments`).

La P1 (Opus) a rendu 0/0/8/9, remédiée au commit `22f426f8b` (liste au Change Log de la fiche). Vérifie que la remédiation
tient et cherche ce qu'elle n'a pas vu. Une question reste au Project Lead (page hors de tout groupe) : ne la tranche pas,
juge seulement si le choix provisoire (a) est sûr à développer.

## Interdits

Lecture seule : tu n'écris **que** ton rapport. Aucune modification de fichier du dépôt, aucun commit, aucun push, aucune
commande `gh` qui écrit, aucun script mutant, aucun build, test, `npm`, `cargo`, `docker`.

## Axes (tous à exercer, et à déclarer exercés ou non)

1. Prémisses au code (`frontend/src/routes/(app)/+layout.svelte`, `navGroups`, stockage du choix) : sites cités par `grep -nF`.
2. Comportement : un seul groupe ouvert, défaut, refermeture, auto-dépliage sans écriture, oubli au changement de chemin,
   lecture de l'ancien format mémorisé, `ontoggle` de réconciliation, accessibilité (AC6).
3. AC et tests : chacun vérifiable, chaque mutation M1 à M7 tuée par un test nommé ; inventaire des sites non résolus
   (E2E existants qui supposent deux groupes ouverts : grep sur `nav-group-`).
4. Manuel : `docs/manual/` contre la fiche et le code, **PDF compris**, aplati avant de greper
   (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) ; fichier partagé avec la 66-1.
5. Recette visuelle, gates, statut de la fiche (elle porte `ready-for-dev` alors que la validation n'est pas finie), décomptes.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e66/66-2/66-2-validation-p2.md` : findings numérotés avec sévérité et preuve ; décompte ;
axes exercés et non exercés. Réponds par le décompte et les findings au-dessus de LOW, en une ligne chacun.

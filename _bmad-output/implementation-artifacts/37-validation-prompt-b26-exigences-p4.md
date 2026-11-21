# Prompt de validation ciblée P4 — exigences FR/NFR de l'Epic 37 (B26)

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*), contexte frais. Worktree `<dépôt>-wt37`
(branche `epic/37-multi-devises`) : n'écris nulle part ailleurs. Fiche indiquée dans ta mission.

## Périmètre

- La remédiation à relire : `git diff f999c5bf 1d6d8cb7 -- <fiche>` (lecture seule) — section `## Exigences` ajoutée
  (FR-37-*, NFR-37-*) : **correctifs de la passe P3** ; lis l’entrée « validation ciblée P3 des exigences » du Change Log.
- Pour chaque exigence : est-elle **vraie** au regard de l'AC **entier** qu'elle cite (lis l'AC en entier) ? Trop large,
  trop étroite, fausse, mal renvoyée ? Un point majeur d'un AC (contrat d'API, refus, garde comptable, sauvegarde,
  audit, RBAC, MCP) manque-t-il de toute exigence ? Une exigence invente-t-elle ce qu'aucun AC ne porte ?
- Vérifie aussi que le récapitulatif de la fiche dans `_bmad-output/planning-artifacts/epic-37-multi-devises.md`
  (section « Exigences (FR/NFR) par story ») concorde.
- Hors périmètre : les AC eux-mêmes.

## Sévérités

MEDIUM : exigence fausse ou contraire à un AC, ou point majeur d'AC sans exigence. LOW : formulation, renvoi, doublon.

## Interdits

N'écris que dans la fiche visée et dans sa ligne du récapitulatif de l'epic, par **Edit** seulement. Aucun script,
aucun `cargo`/`npm`, aucun `git` en écriture.

## Rapport (français)

Findings `<story>-B26-P4-N` (sévérité, exigence, constat avec la ligne de l'AC lue, correctif) ; axes exercés / non
exercés ; décompte. Phase 2 : corrige ; Change Log « 2026-11-15 — validation ciblée P4 des exigences (<modèle>) :
<décompte> ». Message final bref.

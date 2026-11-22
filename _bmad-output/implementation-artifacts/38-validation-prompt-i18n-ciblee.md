# Prompt de validation ciblée — préfixes i18n (constat de la validation de l'Epic 40)

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*), contexte frais, ouvrant un mini-cycle neuf sur des fiches closes de
l'Epic 38 (constat venu de l'Epic 40, comme un CR). Dépôt `<dépôt>`, branche `epic/38-paypal`.

## Périmètre

- Remédiation à relire : `git diff 1933dd59 5b6b0443` (lecture seule) — préfixes des clés i18n de 38-9a (AC-1, AC-10, AC-15),
  38-5 (AC-10) et 38-10 (AC-9).
- Vérifie au code la règle `keyBelongsToFeature` (`frontend/scripts/lint-i18n-ownership.js`) et, pour chaque clé ou
  préfixe nommé : le fichier qui l'emploie est-il bien sous la fonctionnalité dont il porte le préfixe ? un préfixe
  global est-il employé à tort ? `nav-admin-providers` suit-il les clés `nav-*` existantes ? les fiches sœurs (38-9b,
  38-6a, 38-6b, 40-6 en lecture seule dans `../kesh-wt40`) qui reprennent des clés de 38-9a ont-elles un renvoi devenu
  faux ?
- Hors périmètre : le reste des fiches.

## Interdits

N'écris que dans les trois fiches, par **Edit** seulement. Aucun script, aucun `cargo`/`npm`, aucun `git` en écriture.

## Rapport (français)

Findings `<story>-I18N-N` (sévérité, lieu, constat, correctif) ; axes exercés / non exercés ; décompte. Phase 2 :
corrige ; Change Log « 2026-11-15 — validation ciblée des préfixes i18n (<modèle>) : <décompte> » ; inter-fiches signalés.

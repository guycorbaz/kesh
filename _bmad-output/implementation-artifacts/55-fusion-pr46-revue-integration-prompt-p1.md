# Prompt — revue indépendante des défauts d'interaction de la fusion `c8fe4d0ac` (pr/46-rapports-groupes-v2 dans epic/55-navigation)

Tu relis, **en lecture seule**, la fusion de la tête de la PR de l'Epic 46 (`pr/46-rapports-groupes-v2`, `ecf6cf835` : la
46-5 et sa garde `ACCOUNT_PARENT_BANK_LINKED`, les notes de version, `main` avec les Epics 39 et 40) dans la branche de
l'Epic 55 (`epic/55-navigation`, ancienne tête `99b84d602`, bâtie sur l'ancienne tête de la PR `6fc069082`). **Passe
complète, première. Modèle : Opus. Contexte frais.** Langue du rapport : français.

**Lis d'abord** `_bmad-output/implementation-artifacts/55-code-review-brief-commun.md` (interdits, garde-fou `grep -nF`,
forme du rapport) ; il s'applique, sauf la méthode en trois couches, remplacée par les axes ci-dessous.

- Fusion : `c8fe4d0ac` ; compteurs confirmés : `d539e8a47`. Relis-la en diff combiné : `git show --remerge-diff c8fe4d0ac`
  et `git show -c c8fe4d0ac`.
- Ce que l'Epic 55 a changé depuis la base commune : `git diff 6fc069082 99b84d602` ; ce que la PR a apporté :
  `git diff 6fc069082 ecf6cf835`.
- Garde des compteurs relevée : `<journaux>/s55-revue/fusion/gardes-1.log` ; gate frontend :
  `s55-revue/fusion/front/` ; pré-vol fmt et clippy : `s55-revue/fusion/pre-vol/`.
- Rapport : **`<journaux>/s55-revue/fusion/revue-integration-p1-rapport.md`**.

## Axes (chacun exercé ou déclaré non exercé, avec la raison)

1. **Conflits résolus** (sept fichiers : quatre `messages.ftl`, `sprint-status.yaml`, `i18n-keys.test.ts`,
   `i18n-libelle-en-dur.test.ts`) : les deux intentions gardées, aucune clé en double, aucune perdue, commentaires de
   compteur exacts.
2. **Interactions sans conflit de texte** : deux côtés qui changent la même règle, le même code d'erreur, le même texte,
   le même compteur ou le même test dans deux fichiers différents. En particulier : les rapports par groupes (46-x) et
   `hasDocuments` / les liens de compte de l'Epic 55 dans `kesh-report` (journal, grand livre) ; la page des Rapports
   (`routes/(app)/reports/+page.svelte`) et ce que `main` ou la 46-5 y change ; le manuel utilisateur (fusion
   automatique de `user-manual.tex` : sections qui se contredisent, renvois `\ref` cassés) ; `docs/api-external.md`
   (fusion automatique) ; `crates/kesh-api/src/lib.rs` (routes) ; gardes de textes (`textes_coherents`) et le registre
   d'audit ; le sélecteur de compte et les comptes Wise ou Stripe dans le grand livre.
3. **Compteurs** recomptés depuis la source : `sitesTotal`, libellés en dur, migrations (98), registre d'audit et sa
   somme dans le manuel d'administration.
4. **Tests d'une story invalidés par une garde de l'autre** (garde `ACCOUNT_PARENT_BANK_LINKED`, gardes i18n, specs E2E
   des rapports groupés et de la navigation).
5. **Le manuel** : PDF compilé aplati de la tête (`s55-revue/manuel/fusion/user-manual-aplati.txt`) pour les passages
   fusionnés automatiquement.

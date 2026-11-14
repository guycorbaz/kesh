# Prompt — revue de code P2 ciblée de la Story 35-2a (#964, refs #955), une lentille, braquée sur la remédiation de la P1

Passe **ciblée**, une seule lentille, contexte frais, **Sonnet**, en lecture seule. Worktree `/home/gcorbaz/devel/kesh-e352a`, branche
`story/35-2a-refus-pdf`. La P1 (Opus, trois lentilles) a rendu 2 MEDIUM et des LOW (rapports
`/home/gcorbaz/devel/kesh-gate-logs/e352a-review-p1-{B,E,A}.md`) ; les LOW non traités sont tracés (#1096, #1099 à #1104) et ne sont
pas à rapporter de nouveau.

## Périmètre : le seul commit `f934bd64`

`git -C /home/gcorbaz/devel/kesh-e352a show f934bd64` — onze fichiers : trois replis Rust de `refus_pdf` et deux assertions du
`mod tests` (`crates/kesh-api/src/routes/invoice_pdf_service.rs`), trois clés ×4 locales (`messages.ftl`), deux tests d'intégration
(`invoice_pdf_e2e.rs`, `company_address_e2e.rs`), le manuel (`user-manual.tex` et son PDF), le CHANGELOG et le registre
`e35-choix-autonomes-e352a.md` (C-e352a-8, annotation de C-e352a-1). Le compte rendu est la dernière entrée du Change Log de la fiche
`_bmad-output/implementation-artifacts/35-2a-refus-pdf-contrat-details.md` (commit `16b9b78f`).

Il remédie :

- **B1/E2/A1 (MEDIUM)** — les phrases `invoice-pdf-error-{charset,empty,too-long}-company-name` renvoyaient à « Paramètres →
  Organisation », où le nom ne se modifie pas (`settings/+page.svelte` l'affiche en `<dd>`, les routes de `companies.rs` recopient
  `company.name`). Choix prudent : une phrase vraie, sans écran promis (#1097 trace l'absence d'édition).
- **B2/E1 (MEDIUM)** — les témoins de #964 passaient à vide : les phrases `empty-*-postal-code` et `empty-*-city` contiennent toutes
  deux « NPA et localité », si bien qu'une assertion sur ces mots ne distinguait pas les deux. Remède : asserter la proposition
  « le NPA est vide » / « la localité est vide » et l'absence de l'autre.

## Ce que tu vérifies

1. **La prémisse de B1** : relis `crates/kesh-api/src/routes/companies.rs` (`git grep -n 'name' crates/kesh-api/src/routes/companies.rs`)
   et `frontend/src/routes/(app)/settings/+page.svelte` : aucune route, aucun écran, aucun outil MCP ou API REST n'écrit-il le nom de
   la société ? (`git grep -n 'company.name\|companies::update' crates/`.) Si un chemin l'écrit, la phrase « ne se modifie pas encore
   depuis l'interface » est-elle fausse, ou seulement incomplète ?
2. **Les phrases** : dans les **quatre** `messages.ftl`, les trois clés `*-company-name` — chaque phrase dit-elle vrai, est-elle dans
   la langue de son catalogue, garde-t-elle les arguments (`{ $car }`, `{ $point }`, `{ $max }`, `{ $got }` : relis les noms exacts
   dans la version d'avant) ? Les replis Rust sont-ils **égaux** au fr-CH (garde G9 de `crates/kesh-api/tests/textes_coherents.rs`) ?
   Typographie suisse (apostrophe, guillemets, espaces insécables) comme les clés voisines ?
3. **Propagation du symptôme** : `git grep -n 'Organisation' -- crates/kesh-i18n crates/kesh-api docs/manual/fr/*.tex CHANGELOG.md
   docs/api-external.md frontend/src` (et les équivalents `Organization`, `Organisation`, `Organizzazione` des autres locales) : reste-t-il
   un texte qui renvoie à cet écran pour **le nom** de l'entreprise ? Les renvois restants pour l'adresse ou le pays sont légitimes
   (l'écran les modifie) — vérifie-le au code de l'écran.
4. **Les témoins de B2** : `proposition_du_champ_vide` et son assertion (`invoice_pdf_service.rs`, `mod tests`), les tests
   `refus_nomme_le_npa_vide_de_l_entreprise` et `refus_nomme_la_localite_vide_du_client` (`invoice_pdf_e2e.rs`),
   `company_address_e2e.rs:666-674`. Les sous-chaînes par locale (« die plz ist leer », « il npa è vuoto », « the postcode is empty »…)
   existent-elles **réellement** dans les phrases des quatre catalogues, en minuscules (sinon l'assertion rougirait — ou, pire, la branche
   `None` la ferait passer à vide) ? Combien de clés la branche `Some` exerce-t-elle (recompte depuis les catalogues :
   `/usr/bin/grep -c '^invoice-pdf-error-empty-.*-\(postal-code\|city\) =' crates/kesh-i18n/locales/*/messages.ftl`) ? La garde
   `!texte.contains("Organi")` passe-t-elle à vide pour une locale où l'écran porterait un autre nom ?
5. **La preuve par mutation** (MB2a, MB2b) : `kesh-gate-logs/e352a/mut/campagne-p1.log`, `MB2*-avant.log`, `MB2*-apres.log` — survivante
   avant, tuée après, par un test **nommé** ? La restauration est-elle établie (arbre propre, binaire non périmé : le fichier touché
   après restauration) ? Le script `mutations-b2.py` se **lit**, il ne s'exécute pas.
6. **Le manuel et le CHANGELOG** : `user-manual.tex:1251` et l'entrée du CHANGELOG `## [0.17.7]` disent-ils vrai au code, et citent-ils
   #1097 ? Le PDF aplati porte-t-il la phrase (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -o
   '…'`) ?
7. **Les gates** au commit `f934bd64` : test-fast 5331, gate-ci 5338, Vitest 3188, E2E 404/7 KF-029 — chaque chiffre est-il au journal
   (`kesh-gate-logs/e352a/p1-*.log` et suivants), postérieur au commit ? Le Change Log n'affirme-t-il que ce qui a tourné ? Le registre
   annote-t-il C-e352a-1 sans en réécrire le texte d'origine ?

## Interdits absolus

Lecture seule : aucun fichier écrit hormis ton rapport, aucune redirection (`>`, `>>`, `tee`, `2>`, `2>&1`, `> /dev/null` compris), aucun
fichier temporaire. Aucun `rm`, `pkill`, `kill`, aucun `git` qui écrit. Aucun interpréteur ni script (`python3`, `perl`, `node`, `npm`,
`npx`, `bash -c`, `sh -c`, `jq`, `scripts/*` dont **`scripts/prepare-release.sh`**, les scripts de `kesh-gate-logs`, `make`, `latexmk`,
`cargo`, `docker`, `mariadb`, `curl`, **`claude`**, `sed -i`, `vitest`, `playwright`), aucune commande `gh`, aucun accès réseau. Ne lis
rien sous `~/kesh/` ni `~/kesh-test/`. Permis : `git show/diff/log/grep/ls-files`, `/usr/bin/grep`, `sed -n`, `awk` sans redirection ni
`system()`, `cat`, `ls`, `find` sans `-exec`/`-delete`, `wc`, `sort`, `uniq`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`. Toute affirmation de présence ou d'absence s'appuie sur
une sortie copiée au rapport. Un fichier créé par erreur se déclare.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e352a-review-p2-ciblee.md` : findings numérotés (`C1`…), sévérité (CRITICAL/HIGH/MEDIUM/LOW), lieu,
ligne lue copiée, preuve, remède ; **axes exercés et non exercés** ; décompte final recompté depuis la liste.

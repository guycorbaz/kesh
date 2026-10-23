# Epic 31 — Tenir les livres : une sauvegarde qui restaure tout, puis la répétition d'une clôture

**Kickoff : 2026-10-23.** La v0.14.0 est en fonction sur le NAS depuis le 2026-10-20. Le 2026-10-23, Guy décide de
**tenir sa comptabilité réelle** dans Kesh : relevés BCV importés en CAMT.053 à partir de juillet, saisie manuelle de
janvier à juin, relevés UBS en CSV, factures fournisseur avec ou sans PDF. Il ne testera pas le bouclement pour
l'instant, et reprendra la recette de la v0.14.0 à la release suivante.

Dès lors, la prémisse du `CLAUDE.md` — « *no legacy production data to protect* » — tombe. La première chose qui manque
pour des livres réels est une **sauvegarde qui restaure tout** : aujourd'hui, le `.keshbackup` n'emporte aucun
justificatif ni aucun PDF figé ([#503], P1), et une restauration rendrait les écritures sans leurs pièces.

**Cible : v0.15.0.** Sources : les rétrospectives des Epics 29 et 30 (`epic-29-retro-2026-10-23.md`,
`epic-30-retro-2026-10-23.md`, § 7 et § 10 — réponses de Guy), la revue de projet du 2026-10-23 (R10 avéré, R13 ouvert).

⚠️ **Contrepartie BMAD des issues** — chaque issue tient le détail, cet epic le périmètre. Il se clôt quand les issues
ci-dessous sont fermées, la répétition à blanc écrite, et la v0.15.0 publiée.

| Issue / action | Objet | Story |
|---|---|---|
| [#503] (P1) | la sauvegarde emporte les justificatifs et les PDF figés, et Kesh la fait seul, à intervalle régulier | 31-1 |
| C3 (rétro 29) | le conteneur MariaDB de dev a exactement les droits du fichier versionné | 31-2 |
| [#577] (P3), A-C7 | la CI exécute un smoke E2E — ou le `CLAUDE.md` cesse de l'affirmer | 31-2 |
| A2, A3 (rétro 15) | téléchargements Docker authentifiés en CI ; `.svelte-kit/` versionnés retirés | 31-2 |
| A10 (rétro 15) | Rust 1.97.1 | 31-3 |
| ex-29-7 | la répétition à blanc d'une clôture, au manuel d'administration, **exécutée** sur une copie | 31-4 |

## Le décompte, et la règle de release

```sh
gh issue list --state open --limit 500 --json number --jq length
gh issue list --state open --limit 500 --json labels \
  --jq '[.[]|select([.labels[].name]|any(.=="bug" or .=="known-failure"))]|length'
```

| Moment | Issues ouvertes | dont bugs |
|---|---|---|
| Tag v0.14.0 (2026-10-20) | **136** | 49 |
| Kickoff de l'Epic 31 (2026-10-23) | **135** (−#618, −#621, +#695) | 49 |
| **Plafond au tag v0.15.0** | **moins de 136** | — |

Règle de release de Guy et **« on trace toujours »** (`CLAUDE.md` § *Priorités des défauts*) : un point qu'on décide de
ne pas traiter s'ouvre en issue, même s'il fait remonter le compte. Fermer #503 et #577 donne 133.

**Engagements de l'epic** (§ *Priorités des défauts*) : le nombre de bugs baisse (49 au kickoff) ; aucun P1 ne passe
hors TVA — #503 est traité, **#404 et #235 restent** (rapports au-delà de la borne d'exercice, justificatifs sur les
écritures), à nommer à la rétrospective ; la fonctionnalité nouvelle est la sauvegarde automatique.

## Décisions du Project Lead (2026-10-23)

- **Ordre** : rétrospectives des Epics 29 et 30, revue de projet, puis cet epic, **#503 en premier**.
- **#503** : au-delà des fichiers dans le `.keshbackup`, « une **sauvegarde locale automatique complète à partir de
  Kesh**, permettant de complètement restaurer une installation ». La **configuration `.env` reste hors sauvegarde pour
  l'instant** ; à terme, Guy veut que presque toute la configuration passe par l'interface graphique et que le `.env`
  ne porte presque plus rien — **les réglages nouveaux de cet epic se font donc dans l'interface** quand c'est possible,
  pas par une variable d'environnement de plus.
- **La 29-7 est reportée après #503** : la répétition éprouvera la sauvegarde complète.
- **Conteneur de dev** : lui retirer les droits que la production n'a pas.
- **#577** : laissé à l'orchestrateur — recréer le smoke (C-31-0-2).
- **#671** n'entre pas dans cet epic (avertir et demander à l'utilisateur, non planifié) ; consigne d'usage d'ici là :
  une transaction déjà comptabilisée se *rejette*.
- **Autonomie** : « continue de manière autonome » — jusqu'à l'image Docker v0.15.0 publiée, comme les epics
  précédents ; recette par Guy ensuite.
- **Boucles de revue** : closes seulement sur une passe à **0 MEDIUM+** (décision Q3 c, versée au `CLAUDE.md`).

## Les stories

Chaque story tranche ses points en autonomie, au registre `epic-31-choix-autonomes.md` (ou un registre propre à la
story, versé par union), et suit `docs/agents-orchestration.md` — complété par la 31-0. **Périmètre minimal dès la
fiche** ; un défaut voisin s'intègre s'il est mécanique, sinon il s'ouvre. **Recette visuelle** : captures, thème clair.

### 31-0 — Kickoff et actions des rétrospectives (doc seulement)

Ce fichier, le registre de l'epic, la section `epic-31` de `sprint-status.yaml` ; et ce que les rétrospectives 28 à 30
laissaient « appliqué par les prompts, non versé » : au `CLAUDE.md` la clôture d'une boucle à 0 MEDIUM+, la règle de
release et « on trace toujours », les mots-clés de fermeture, la recette visuelle (thème clair) ; à
`docs/agents-orchestration.md` le défaut voisin, le symétrique client, les privilèges de la base, les lentilles sans
fichier. Sorts écrits des actions anciennes : A1, A7, A-C2 **abandonnées** ; A2, A3, A10 **reconduites** (31-2, 31-3) ;
A6 **répondue** (Hyper Backup arrête le paquet MariaDB avant la sauvegarde) ; A12 **faite** (jalon *Épique 15* fermé,
ses issues au backlog).

### 31-1 — Une sauvegarde qui restaure tout, faite par Kesh ([#503], P1)

Deux volets, que la fiche établit au code puis découpe (règle de découpage préventif : l'export, l'import, le stockage
des pièces, un planificateur, un écran d'administration et les manuels) :

1. **Les fichiers dans la sauvegarde** — critères de l'issue : le `.keshbackup` contient tous les fichiers référencés
   par la base (justificatifs sous `KESH_DOCUMENTS_DIR`, PDF figés), nommés par leur empreinte SHA-256, vérifiés à
   l'import et restaurés avant de rendre la main ; une sauvegarde antérieure, sans fichiers, reste importable ; le
   volume (`KESH_ADMIN_IMPORT_MAX_MB`, export en flux) ; **la restauration sur un système neuf, de bout en bout**
   (installation vierge, `/setup`, import, tout revient — comptabilité, utilisateurs, justificatifs et PDF
   téléchargeables), **testée en E2E** et décrite pas à pas au manuel d'administration ; ce qui reste hors du fichier
   (le `.env`) dit à l'écran et au manuel.
2. **La sauvegarde automatique** — Kesh écrit seul, à intervalle régulier, une sauvegarde complète dans un dossier
   local (celui de `KESH_ADMIN_BACKUP_DIR` ou un sous-dossier), avec une **rétention** bornée ; réglage dans
   l'interface (*Administration*), état de la dernière sauvegarde visible, échec signalé. La fiche tranche : fréquence
   proposée, rétention, ce qui se passe quand le disque est plein, concurrence avec une sauvegarde manuelle ou un
   import, instantané cohérent (`ReadSnapshot`, Story 28-3).

**`kesh-db` touché probable : gate complet même en boucle.** Migration éventuelle (réglages) : la *Migration breaking
policy* s'applique ; une migration non breaking empêche quand même le retour à la 0.14.0 (§ P1). Manuel
d'administration (PDF), i18n quatre locales, `api-external.md` si l'API change.

### 31-2 — L'outillage dit vrai : droits de dev, smoke E2E en CI, Docker authentifié, `.svelte-kit/`

Petite story d'outillage, quatre volets indépendants :

1. **Droits du conteneur de dev (C3).** ⚠️ Prémisse à établir : au kickoff, `SHOW GRANTS FOR 'kesh'@'%'` sur
   `kesh-mariadb-dev` ne rend **ni** les droits complets décrits par la rétrospective 29, **ni** ceux de
   `scripts/mariadb-init/01-dev-grants.sql` (seulement `kesh`.*). La fiche établit quel utilisateur emploient les
   suites (`DATABASE_URL` des tests, `#[sqlx::test]`, E2E), d'où vient l'écart (montage de l'init depuis un autre
   worktree ? cf. journal du 2026-10-15), et rend le montage fiable ; puis gate complet pour voir ce qui rougit.
2. **Smoke E2E en CI ([#577]).** Un job, une spec : login puis un appel API authentifié ; MariaDB service, seed,
   un navigateur ; quelques minutes. Le `CLAUDE.md` et `docs/ci.md` décrivent alors le dépôt.
3. **Docker Hub authentifié (A2)** : `credentials` sous `services: mariadb` de `ci.yml` et `release.yml` (les secrets
   `DOCKERHUB_USERNAME` / `DOCKERHUB_TOKEN` existent déjà pour `release.yml`) ; la note A-C9 du `CLAUDE.md` est retirée.
4. **`.svelte-kit/` (A3)** : les six fichiers versionnés à la racine retirés, `/.svelte-kit/` au `.gitignore` racine.

### 31-3 — Rust 1.97.1 (A10)

`rust-toolchain.toml` à 1.97.1, nettoyage des cibles, gate complet (nouveau clippy), CI verte.

### 31-4 — La répétition à blanc d'une clôture (ex-29-7, doc)

**Après la 31-1.** Cahier des charges inchangé — commentaire de la clé `29-7-repetition-a-blanc` de
`sprint-status.yaml` : écrite **en l'exécutant réellement** sur la station avec une sauvegarde **synthétique**, jamais
sur la base du NAS ; ne pas se fier au nom du compose ; `.env` de la copie depuis `.env.example` ; montages absolus
absents ; URL au tag `v<keshVersion>` (G19). La répétition importe désormais la **sauvegarde complète** de la 31-1.

### Release v0.15.0

`scripts/prepare-release.sh`, macros des manuels, URL au tag ; **et les textes qui changent avec la comptabilité
réelle** (décision de Guy, « à la release suivante ») : le `CHANGELOG` cesse de dire la version « pas destinée à tenir
une comptabilité réelle », et le `CLAUDE.md` récrit le paragraphe « *Deployed is not the same as keeping the books* »
et le raisonnement qui s'appuie sur « *no legacy production data to protect* » (R13).

## Hors périmètre

[#671] (avertir et demander, non planifié) ; [#404] et [#235] (P1, à nommer à la rétrospective) ; la configuration du
`.env` par l'interface (souhait de Guy à terme — cet epic n'y ajoute simplement pas de variable) ; [#695] (pagination
au-delà de 500).

[#503]: https://github.com/guycorbaz/kesh/issues/503
[#577]: https://github.com/guycorbaz/kesh/issues/577
[#671]: https://github.com/guycorbaz/kesh/issues/671
[#404]: https://github.com/guycorbaz/kesh/issues/404
[#235]: https://github.com/guycorbaz/kesh/issues/235
[#695]: https://github.com/guycorbaz/kesh/issues/695
[#618]: https://github.com/guycorbaz/kesh/issues/618
[#621]: https://github.com/guycorbaz/kesh/issues/621

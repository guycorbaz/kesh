# Epic 28 — registre des choix autonomes

*Registre tenu par union (jamais par écrasement). Chaque choix : contexte, option retenue, alternatives, réversibilité.
Fichier d'epic : `planning-artifacts/epic-28-recette-v0-13-2.md`. Consignes : `docs/agents-orchestration.md`.*

---

## Story 28-0 — Kickoff et actions reportées (doc seulement)

Choix faits au kickoff (2026-10-13), en autonomie, sur `6f971d76` (rétrospective de l'Epic 27).

### C-28-0-1 — Les consignes des agents dans un document propre, non dans `docs/testing.md`

- **Contexte** : Q5 de la rétrospective de l'Epic 27 (B6 de l'Epic 26, A8 de l'Epic 15) — verser au dépôt les
  consignes des agents et le verrou de gate. Deux lieux possibles : une section de `docs/testing.md`, ou un document
  à part.
- **Retenu** : `docs/agents-orchestration.md`. Les consignes portent pour moitié sur autre chose que les tests —
  autonomie et registre des choix, interdits (`git push`, `gh`, `prepare-release.sh`), lentilles de revue, édition
  du code, données réelles — et s'adressent à un autre lecteur (l'orchestrateur qui rédige un prompt d'agent).
  Ce qui relève des tests y **renvoie** à `docs/testing.md` (recette E2E, base jetable) plutôt que de le recopier.
  Le `CLAUDE.md` y renvoie en une ligne, § *Plafonds mémoire*, là où le verrou prolonge `mem-guard.sh`.
- **Écartées** : section de `docs/testing.md` (le document grossirait d'un sujet qui n'est pas le sien) ; `_bmad/`
  (réservé au cadre BMAD installé, réécrit par une réinstallation).
- **Réversible** : oui (déplacement de fichier).

### C-28-0-2 — Aucun chemin personnel : le répertoire des journaux nommé `$GATE_LOGS`

- **Contexte** : la consigne demande de ne garder aucun chemin personnel inutile. Le verrou vit dans
  `kesh-gate-logs/.gate.lock`, répertoire voisin des worktrees sur la station.
- **Retenu** : convention `$GATE_LOGS` définie en tête du document (« voisin de `kesh/` et des `kesh-<clé>/` ») ;
  le `CLAUDE.md` écrit `<kesh-gate-logs>/.gate.lock`. Les références à la mémoire de l'orchestrateur, au registre
  « epic-15 » et au fichier de reprise de l'Epic 26 sont généralisées (`epic-<N>-choix-autonomes.md`).
- **Écartées** : le chemin absolu de la station (personnel, non portable) ; une variable d'environnement réellement
  lue par un script (aucun script ne prend le verrou aujourd'hui : ce serait du code, hors du périmètre doc).
- **Réversible** : oui.

### C-28-0-3 — B7 : la recette E2E du `CLAUDE.md` est un défaut, corrigé par recopie intégrale de `docs/testing.md`

- **Contexte** : `CLAUDE.md:180` portait `KESH_JWT_SECRET='<32+ octets>'` et `KESH_ADMIN_PASSWORD='<12+ caractères>'`.
  `is_template_placeholder` (`crates/kesh-api/src/config.rs`) refuse au démarrage toute valeur de la forme `<…>` :
  recopiée, la recette ne démarre pas. Le bloc omettait aussi `KESH_INBOX_DIR`, `KESH_DOCUMENTS_DIR` et les quatre
  `KESH_SMTP_*` (cinq specs rouges hors sujet), et `KESH_TEST_MODE` côté runner.
- **Retenu** : c'est un défaut ; le bloc est remplacé par celui de `docs/testing.md` § *Prérequis Playwright local*,
  déclaré comme faisant foi, valeurs factices concrètes, avec une note datée qui dit pourquoi elles ne doivent pas
  redevenir des gabarits.
- **Écartées** : ne remplacer que les deux gabarits (la recette serait restée incomplète — c'est la recette du
  `CLAUDE.md` qui avait induit la troisième récidive du 2026-09-16) ; renvoyer seulement à `docs/testing.md` sans
  bloc (la section explique un piège sur ce bloc même).
- **Réversible** : oui.

### C-28-0-4 — B4 versé au `CLAUDE.md`, § *Recompter ses propres comptes rendus*

- **Contexte** : B4 de la rétrospective de l'Epic 27 prévoyait la consigne en mémoire, « puis au `CLAUDE.md` sur
  accord ». Guy a répondu à la Q5 que les actions reportées entrent dans cette release.
- **Retenu** : un paragraphe sous la règle existante (ne jamais réécrire l'entrée d'une passe passée ; corriger
  par annotation datée ce qui était faux à sa date), et la même consigne au § 3 de `docs/agents-orchestration.md`.
- **Écartée** : attendre un accord explicite sur le seul B4 (la Q5 couvre les actions reportées).
- **Réversible** : oui.

### C-28-0-5 — A-C9 (Epic 15) versé : la CI dépend de Docker Hub

- **Contexte** : amendement proposé, « à retirer si A2 est faite ». Vérifié au kickoff : `image: mariadb:10.11` dans
  `ci.yml:50` et `release.yml:21`, aucun `credentials`.
- **Retenu** : une phrase au § *Quand sauter*, avec sa condition de retrait.
- **Réversible** : oui.

### C-28-0-6 — « dix P1 de la TVA » ; la ligne datée du triage du 2026-10-08 reste à « 9 »

- **Contexte** : B6 de l'Epic 27. `gh issue list --milestone "Vague 2 — …" --state open --label P1` rend 10 (#390 à
  #397, #401, #525). Le grep de la valeur trouve aussi « 14 P1 (9 dans le jalon de la TVA) » au paragraphe « Point
  de départ, au triage du 2026-10-08 ».
- **Retenu** : le paragraphe du report passe à dix, avec la commande ; la ligne du triage reste à 9 — relevé daté,
  juste à sa date (B4).
- **Réversible** : oui.

### C-28-0-7 — Actions reportées non traitées par la 28-0, et pourquoi

Ne demandent pas qu'un commit de documentation, ou relèvent d'une décision qui n'est pas la mienne :

| Action | Sort | Raison |
|---|---|---|
| A-C2 (Epic 15) — D5 bis, une dérogation vaut pour une passe | **non versé** | change une règle de découpage ; la rétrospective de l'Epic 27 (§ 3.2) relève que D5 « se lit encore de deux façons » : à trancher par Guy, à la rétrospective de l'Epic 28 |
| A-C3, A-C4, A-C5 (Epic 15) | **versés dans `docs/agents-orchestration.md`** (§ 3, 4, 6), non au `CLAUDE.md` | ce sont des consignes d'orchestration ; le `CLAUDE.md` y renvoie |
| A-C7 / A9 (Epic 15) — #577, smoke E2E en CI | **listé** | décision de Guy (recréer le smoke ou réécrire le texte) ; le `CLAUDE.md` et `docs/ci.md` portent déjà le constat |
| A-C8 (Epic 15) — prémisse P1 de la *Migration breaking policy* | **listé** | fait à vérifier à l'exécution (`MIGRATOR.ignore_missing`, refus `VersionMissing`) avant de l'écrire ; candidat à la 28-4, qui ajoute une migration |
| A2 (Epic 15) — authentifier Docker Hub en CI | **listé** | modifie `ci.yml` / `release.yml` (code de CI), décision de Guy |
| A3 (Epic 15) — six `.svelte-kit/` versionnés à la racine | **listé** | retrait de fichiers + `.gitignore`, pas de la documentation ; candidat à un `chore` |
| A7 (Epic 15) — combler les comptes rendus | **listé** | lourd ; ne se fait que par annotations datées (C-28-0-4) ; lot dédié |
| A10 (Epic 15) — Rust 1.97.1 | **listé** | code et gate complet |
| A1, A6 (b, c), A12 (Epic 15) | **listé** | décisions ou réponses de Guy (choix de schéma, Synology, jalon de l'Epic 15) |
| A4 (Epic 15) ; B3 (Epic 26) / B5 (Epic 27) — priorités de #618 et #621, jalons de #602, #606, #612 | **listé, demandé à l'orchestrateur** | écritures `gh` (labels, jalons), interdites à l'agent ; #618 et #621 portent toujours `triage` sans priorité au kickoff |
| B2 (Epic 27) — recette visuelle au `CLAUDE.md` | **reporté à la rétrospective de l'Epic 28**, comme l'écrit l'action | appliquée dès cet epic par le fichier d'epic (§ *Les stories*) |
| B4 (Epic 26) / Q6 (Epic 27) — #530 | **fait par Guy** | fermée le 2026-10-13 |
| Q1 (Epic 26) / Q2 (Epic 27) — révisions de #620, #621 | **fait par Guy** | supprimées le 2026-10-13 |

- **Réversible** : oui (chaque ligne peut être reprise par une story ou un `chore`).

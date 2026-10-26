# Prompt — validation P1, Story 32-6c (le journal du filtre d'adresse du serveur MCP)

*Versionné le 2026-10-25. Passe **complète** (première passe). **Deux lentilles, R et F**, chacune en contexte frais,
**Sonnet**. Lecture seule stricte, **sans `gh`**, **sans interpréteur**, **sans fichier temporaire**.*

Dépôt `/home/gcorbaz/devel/kesh-32-6c`, branche `story/32-6c` (partie de `origin/main` `c5287dec` : 32-6a, 32-6b, 32-8b0
et 32-8a fusionnées). Fiche : `_bmad-output/implementation-artifacts/32-6c-journal-filtre-adresse.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-326c.md` (C-32-6c-1 à 10) ; entrée § *32-6c* de
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` ; clé `32-6c-journal-filtre-adresse` de
`_bmad-output/implementation-artifacts/sprint-status.yaml`. Contexte à lire : la fiche 32-6
`_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md` (AC-a7 à AC-a9, AC-b4, § *Ce que les gardes ne voient
pas*, § *Le manuel (axe de revue)*, points 2 et 2 bis pour Guy) et son registre `epic-32-choix-autonomes-326.md`
(C-32-6-4, 5, 6) ; l'epic, § *Réponses de Guy (2026-10-25)*. Règles : `CLAUDE.md` (§ *Review Iteration Rule* et *La
passe ciblée* — une boucle ne se clôt que sur une passe sans finding au-dessus de LOW —, *Inventorier les sites NON
RÉSOLUS*, *Règle de splitting préventif*, *Le prompt d'une passe doit NOMMER le manuel*, *Recompter ses propres
comptes rendus*, *Recette visuelle*), `docs/agents-orchestration.md`.

**Contraintes de la story, à tenir pour vraies et à contrôler** : but **diagnostique** (Guy, 2026-10-25 : « on
vérifiera lors de la mise à jour : ajoute le nécessaire dans les logs au cas où ») ; **aucune clé, aucun en-tête
`Authorization`, aucun corps, aucune donnée comptable** au journal ; **le plus simple qui suffit** ; **un événement par
décision**, `warn` au refus, `debug` à l'admission ; **rien ne change pour le client** (route absente, réponses fixes) ;
au plus cinq modules ; aucune donnée réelle (adresses de documentation ou privées génériques). Ce que la fiche
**prescrit** au développement n'existe pas encore : son absence n'est pas un défaut.

## Les deux axes obligatoires, pour les deux lentilles

- ⛔ **Chaque ligne citée se rouvre.** Pour chaque `fichier:ligne` de la fiche et du registre (faits 1 à 12, AC,
  Dev Notes, § *À recontrôler*), ouvre le site (`sed -n`, `grep -nF`) et **copie la sortie**. Une citation que le
  fichier ne porte pas est un finding. Les comptes écrits (« 6 lignes » du fait 2, sorties de `grep` du fait 12) se
  **rejouent** : `grep -n "tracing::" crates/kesh-api/src/mcp/reseau.rs crates/kesh-api/src/mcp/transport.rs`, `grep -n
  "RUST_LOG" docs/manual/fr/admin-manual.tex`.
- ⛔ **Chaque affirmation sur une dépendance se prouve à sa source**, version du `Cargo.lock`
  (`~/.cargo/registry/src/*/`) : `tracing-subscriber-0.3.23/src/filter/env/directive.rs` (correspondance de cible par
  préfixe, fait 8), `tracing-subscriber-0.3.23/src/fmt/format/mod.rs` (guillemets des champs texte, fait 9),
  `tracing-core-0.1.36/src/field.rs` (booléens et entiers), `axum`/`http` pour `HeaderMap::get_all`, `contains_key`,
  `HeaderValue::as_bytes`. Une affirmation dont tu ne trouves pas la ligne est un finding.

## Lentille R — la spécification contre le code, les fiches et le manuel

Axes, tous à exercer :

1. **Les faits 1 à 12** : chacun est-il vrai au code de `c5287dec` ? En particulier le fait 2 (« ni l'admission ni le
   refus ne laissent de trace ») : cherche **tout** évènement émis sur le chemin d'une requête refusée par le filtre
   (`transport.rs`, `routes/mcp.rs`, `lib.rs`, middleware de `kesh-api`, `tower_http::trace` s'il est monté). Un
   évènement oublié rend la story en partie inutile ou en double.
2. **La cohérence avec la 32-6** : la fiche contredit-elle AC-a7, AC-a8, AC-a9, AC-b4, C-32-6-4 à 6, ou la décision de
   Guy sur la garde 2 bis ? L'AC-3 (client calculé filtre coupé) et l'AC-4 (`Ecartee` sans évènement) laissent-ils la
   décision **exactement** celle de `c5287dec` sur les cas (a) à (s) et (n') ? L'AC-6 est-il vérifiable tel qu'écrit ?
3. **L'évènement** (AC-1, AC-2, AC-5) : les onze champs suffisent-ils aux six éléments demandés par Guy ? Un champ
   porte-t-il, directement ou par sa mise en forme (`%`, `?`), une valeur reçue du client ? `pair` et `client` sont-ils
   bien des adresses **analysées** ? Le comptage « sur les octets » est-il juste pour une ligne non ASCII, vide, ou
   terminée par une virgule ?
4. **Le manuel — `.tex` et PDF** (axe obligatoire de `CLAUDE.md`) : l'AC-9 et l'AC-10 décrivent-ils ce que le code
   prescrit fera, sans promesse non vérifiée (DSM) ? La place (`admin-manual.tex:3017-3027`, `:638-661`) est-elle juste ?
   Une phrase existante devient-elle fausse avec la story (`:652-655` `X-Real-IP`, `:696`, § `sec:logs-fichier`
   `:939-960`, § `sec:traefik`) ? Contrôle le PDF actuel aplati : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n'
   ' ' | tr -s ' ' | grep -oiE ".{60}(X-Real-IP|RUST_LOG|docker compose logs).{80}"` — copie la sortie. La commande
   `docker compose up -d` suffit-elle à appliquer un `RUST_LOG` changé dans `.env` (`docker-compose.yml:91`) ?
5. **Les décomptes** : recompte à la source — 17 AC (11 + 6), 6 tâches, 16 mutations, 12 cas à l'AC-T1, C-32-6c-1 à 10,
   3 modules — par `grep -c` sur la fiche et le registre, sortie copiée. La clé de `sprint-status.yaml` se lit-elle
   (`clé: valeur  # commentaire`, indentation de deux espaces, aucune ligne voisine touchée) ?

## Lentille F — faisabilité, simplicité, tests et fausses prémisses

1. **Le plus simple qui suffit** : un seul évènement par décision suffit-il au diagnostic de Guy à la mise à jour du
   NAS ? Un champ est-il superflu, ou en manque-t-il un sans lequel un cas du tableau de lecture de l'AC-9 ne se
   tranche pas ? Le choix « par société, dans `decider` » (C-32-6c-2) contre « par requête, au transport » tient-il ?
2. **Les tests** (AC-T1 à AC-T6) : chacun est-il réalisable avec `tests/common/capture_trace.rs` et
   `tests/common/mcp.rs` tels qu'ils sont (lis-les) ? La capture voit-elle les évènements de `decider` (fil, `oneshot`) ?
   L'égalité de chaîne complète est-elle constructible (format ` nom=valeur` de la capture, message, ordre) ? Les
   adresses des douze cas donnent-elles le verdict écrit, contre `RESEAUX_DEFAUT` (`tests/common/mcp.rs:162`) et la
   lecture de droite à gauche ? AC-T3 a-t-il un témoin qui l'empêche de passer à vide ? AC-T6 est-il réalisable dans
   `logging.rs` (un abonné local, `with_default`, une couche de capture, `target:` explicite) ?
3. **Les mutations M1 à M16** : chacune est-elle réalisable et tuée par le test **nommé** ? Une passerait-elle à vide
   (M8, M9 et le choix de `172.18.0.1` dans `172.16.0.0/12` ; M14 au transport ; M16) ? Une propriété de l'AC sans
   mutation ?
4. **La garde des secrets** (AC-5, AC-T4) : ferme-t-elle la classe « valeur d'en-tête au journal » **sur ce site** ?
   Que ne voit-elle pas, et le § *Ce que les gardes ne voient pas* le dit-il ? Un `HeaderValue` reçu par `oneshot`
   est-il marqué sensible (sa forme `Debug` masquerait alors la valeur et ferait passer M11 à vide) — prouve-le à la
   source de `http`.
5. **Le comportement inchangé et l'oracle** : avancer le calcul du client (Dev Notes, esquisse) peut-il changer le
   verdict, une réponse, ou le temps de façon observable ? Le journal peut-il devenir un oracle pour le client ?
6. **Les points à soumettre à Guy** : chacun est-il posé en question, avec une recommandation ? En manque-t-il un (le
   niveau `warn` et l'amplification, la valeur des en-têtes, le filtre coupé, DSM, la place au manuel) ? L'un d'eux
   est-il déjà tranché par une décision de Guy citée dans l'epic ?

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt ou de la dépendance,
preuve (commande et sortie copiée, `grep -nF` pour toute affirmation de présence ou d'absence), correction proposée.
Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés ET non exercés** (R1 à R5 ou F1 à F6, et
les deux axes obligatoires) — sans elle, « 0 finding » ne compte pas comme passe. Rapport écrit **d'un seul jet**, un
seul fichier par lentille, avec l'outil d'écriture de fichier : `/home/gcorbaz/devel/kesh-gate-logs/326c-validate-p1-R.md`
et `/home/gcorbaz/devel/kesh-gate-logs/326c-validate-p1-F.md`. Dernier message : chemin, bilan par sévérité, une ligne
par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`** et **`scripts/test-fast.sh`**), `make`, `latexmk`,
`cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, `wget`, **`python3`, `python`, `perl`, `ruby`, `node`, `awk`** et
tout autre interpréteur, `mariadb`/`mysql`, `sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`, et
**`gh` sous toutes ses formes** (même en lecture). `pdftotext` vers la sortie standard seulement. Aucune donnée réelle
de Guy (rien de `~/kesh-test/`). Le dépôt principal `/home/gcorbaz/devel/kesh` est hors de ton périmètre. Autorisés :
lecture, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`, `pdftotext … -`, et les filtres en lecture seule `tr`,
`sort`, `cut`, `head`, `tail`, `wc`, `uniq` vers la sortie standard.

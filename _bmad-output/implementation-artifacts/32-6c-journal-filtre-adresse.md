# Story 32.6c : le journal du filtre d'adresse du serveur MCP

Status: ready-for-dev

*Fiche rédigée le 2026-10-25, en autonomie (Opus 5.5), worktree `kesh-32-6c`, branche `story/32-6c` partie de
`origin/main` `c5287dec` (32-6a, 32-6b, 32-8b0 et 32-8a fusionnées). Registre : `epic-32-choix-autonomes-326c.md`
(C-32-6c-1 à C-32-6c-10). Clé courte : `326c`. Issue de l'epic : #723 (CR « Serveur MCP intégré à Kesh »), en `refs`
seulement. Les numéros de ligne cités sont ceux de `c5287dec`.*

⚠️ **Story de diagnostic, sans changement de comportement.** Elle ajoute au journal du serveur ce qu'il faut pour
vérifier, à la mise à jour du NAS, ce que le filtre d'adresse de `/mcp` voit derrière le proxy inversé de DSM. Aucune
réponse envoyée au client ne change ; aucune migration ; aucun contrôle visible.

## Story

En tant que Guy, qui mettra à jour Kesh sur son NAS Synology derrière le proxy inversé de DSM, je veux lire dans le
journal du serveur, pour chaque requête vers `/mcp`, l'adresse du pair TCP, la présence et la forme des en-têtes de
mandataire, le client que le filtre retient et son verdict — sans aucune clé, aucun en-tête d'authentification, aucun
corps ni aucune donnée comptable — afin de savoir si le filtre d'adresse filtre vraiment, et de régler les mandataires
de confiance en connaissance de cause.

## Origine

- **Guy, le 2026-10-25** : derrière le proxy inversé de DSM, le filtre d'adresse de `/mcp` ne filtre que si DSM pose
  `X-Forwarded-For` ; rien ne l'établit. « On vérifiera lors de la mise à jour : ajoute le nécessaire dans les logs au
  cas où. »
- **Fiche 32-6** (`32-6-endpoint-mcp.md`) : § *Ce que les gardes ne voient pas*, « Le filtre d'adresse voit le pair que
  voit Kesh, non le client réel » — **non vérifié sur le NAS** ; point 2 pour Guy : « quelle adresse voit-il pour un
  client (le journal du serveur la donnera après la 32-6a) ? ». **Le journal ne la donne pas** : fait 2 ci-dessous.
- **C-32-6-4** (`epic-32-choix-autonomes-326.md:57-64`) : « Le refus est au journal du serveur (`tracing`), jamais en
  base. » Et l'AC-b4 de la 32-6b (`32-6-endpoint-mcp.md:786-790`) : les refus du transport vont « au **journal du
  serveur** seulement ». **Pour le refus du filtre, ce journal n'existe pas** (fait 2) : la 32-6c tient la promesse
  écrite par C-32-6-4.
- **Décision de Guy du 2026-10-25** (relayée par l'orchestrateur) : la garde « 2 bis » (refuser un pair non déclaré
  qui envoie des en-têtes de mandataire) est écartée, la sécurité relevant de l'infrastructure
  (`32-6-endpoint-mcp.md:1359-1361`). La 32-6c **n'y revient pas** : elle observe, elle ne décide rien de plus.

## Ce que le code dit (relevé au code le 2026-10-25, `c5287dec`)

1. **La décision du filtre est une fonction pure, par société** : `mcp::reseau::decider(s, pair, en_tetes) -> Decision`
   (`crates/kesh-api/src/mcp/reseau.rs:104-140`), appelée **une fois par société activée** et par requête, sans court-
   circuit, par le transport (`transport.rs:91-94` : `.filter(|s| reseau::decider(s, pair, requete.headers()) ==
   Decision::Admis)`). Trois issues : `Admis`, `Refuse`, `Ecartee` (`reseau.rs:91-99`).
2. **Aujourd'hui, ni l'admission ni le refus ne laissent de trace.** `decider` n'émet que deux évènements, qui ne
   portent ni le pair ni le client : `tracing::error!` pour une entrée de `trusted_proxies` qui ne s'analyse pas
   (`reseau.rs:110-114`, issue `Ecartee`) et `tracing::debug!` pour une entrée ignorée de `allowed_networks`
   (`reseau.rs:129-132`). Le transport, quand aucune société ne sert le client, rend la route absente sans rien écrire
   (`transport.rs:95-97`). Le `warn!` de `reseau.rs:161-164` parle de `KESH_PUBLIC_BASE_URL` (au démarrage), non du
   filtre. `grep -n "tracing::" crates/kesh-api/src/mcp/reseau.rs crates/kesh-api/src/mcp/transport.rs` → 6 lignes :
   `reseau.rs:110`, `:129`, `:161`, `transport.rs:87`, `:121`, `:138` — aucune ne nomme une adresse.
3. **Le filtre désactivé ne calcule pas le client** : `if !s.address_filter_enabled { return Decision::Admis; }`
   (`reseau.rs:119-121`) précède `client(...)` (`:122`). Le mandataire est lu **avant** l'état du filtre (`:105-118`,
   P7-H3 de la 32-6a).
4. **Le client** : `reseau::client(pair, en_tetes, mandataires) -> Option<IpAddr>` (`reseau.rs:63-87`). Pair hors des
   mandataires → `Some(pair)`, `X-Forwarded-For` non lu (`:65-67`) ; sinon toutes les lignes, coupées aux virgules, de
   droite à gauche ; une valeur non ASCII ou une entrée qui n'est pas une adresse nue → `None` (`:70`, `:72`). `None`
   donne `Refuse` filtre activé (`:122-124`). `X-Real-IP` et `Forwarded` ne sont **jamais** lus (doc du module,
   `:4-8` ; C-32-6-5).
5. **Le pair est normalisé deux fois** : `pair.ip().to_canonical()` au transport (`transport.rs:81`) et
   `pair.to_canonical()` dans `client` (`reseau.rs:64`).
6. **« Aucune plage » n'est pas « filtre désactivé »** : filtre activé et `allowed_networks` vide → **personne** n'est
   admis (AC-a8, `32-6-endpoint-mcp.md` ; test (o), `tests/mcp_reseau_e2e.rs:264-268`). Le filtre désactivé, c'est
   `address_filter_enabled = FALSE` seulement.
7. **Le serveur désactivé ne décide rien** : `serveur_mcp::reglages_actifs` ne rend que les sociétés `enabled = TRUE`
   (`services/serveur_mcp.rs:38-47`) ; sans société activée, `servies` est vide sans qu'aucun `decider` ne tourne
   (`transport.rs:84-97`).
8. **Le niveau de journal** : `RUST_LOG` si non vide, sinon `info` (`logging.rs:129-131`) ; `build_log_filter` passe la
   chaîne à `EnvFilter::new` et ajoute `sqlx=warn` sauf directive `sqlx` explicite (`logging.rs:104-120`). Les
   composes transmettent `RUST_LOG: ${RUST_LOG:-info}` (`docker-compose.yml:91`, `docker-compose.prod.yml:109`) ;
   `.env.example:245-253` le documente. Une directive de cible se compare au **préfixe** de la cible de l'évènement
   (`tracing-subscriber-0.3.23/src/filter/env/directive.rs:246` : `meta.target().starts_with(&target[..])`) ; la cible
   d'un évènement de `reseau.rs` est `kesh_api::mcp::reseau` (le test `n_prime_…` l'exige :
   `tests/mcp_reseau_e2e.rs:302`).
9. **La mise en forme** : le journal standard (`fmt::layer()`, `logging.rs:135`) cite les champs texte entre guillemets
   — `record_str` → `record_debug(field, &value)` hors `message` (`tracing-subscriber-0.3.23/src/fmt/format/mod.rs:1264-1273`)
   — et rend `%valeur` par `Display`, sans guillemets ; booléens et entiers passent par `record_debug`
   (`tracing-core-0.1.36/src/field.rs:296-297`, `:311-312`). La capture des tests, elle, écrit ` nom=valeur` sans
   guillemets pour tous (`tests/common/capture_trace.rs:30-38`). **Un exemple de ligne au manuel se copie donc d'une
   sortie réelle**, non de la capture (C-32-6c-8).
10. **La capture des tests** : `CaptureTrace::installer()` retient tous les évènements, tous niveaux, sous la forme
    `"<NIVEAU> <cible> champ=valeur…"` (`tests/common/capture_trace.rs:40-47`), par fil (`:9-14`) ; elle sert déjà à
    l'issue `Ecartee` (`tests/mcp_reseau_e2e.rs:289-307`) et à `jeton_jamais_journalise`
    (`tests/mcp_transport_e2e.rs:1299-1336`).
11. **Précédent d'adresse au journal** : la connexion journalise l'adresse du client en `info!` et `warn!`
    (`routes/auth.rs:190`, `:229`, `:239`, `:652` : `ip = %ip`), dont un `warn!` **par requête** une fois la limite de
    débit atteinte (`:190`).
12. **Le manuel** : DSM, étape 4 (`docs/manual/fr/admin-manual.tex:638-661`) fait poser `X-Forwarded-Proto` et
    `X-Real-IP` (`:652-655`) — **pas** `X-Forwarded-For` ; or `/mcp` ignore `X-Real-IP`. Journaux DSM : `:696`.
    § *Monitoring et logs* (`:3017`), § *Logs applicatifs* (`:3019-3027`, `docker compose logs`). **Aucune ligne ne parle
    de `RUST_LOG`** (`grep -n "RUST_LOG" docs/manual/fr/admin-manual.tex` → `:1643`, `:1885`, deux `grep` d'environnement)
    et **aucune ne décrit `/mcp`** (la section *Le serveur MCP* est à la 32-7, `backlog`).

## Critères d'acceptation

Tests dans `crates/kesh-api/tests/mcp_reseau_e2e.rs` (par `CaptureTrace`, pair simulé par `MockConnectInfo`, comme les
tests (a) à (s)) et dans `crates/kesh-api/src/logging.rs` (`mod tests`). Adresses d'exemple : plages de documentation
(`192.0.2.0/24`, `198.51.100.0/24`, `2001:db8::/32`) et adresses privées génériques ; aucune donnée réelle.

### A — L'évènement de décision

1. **AC-1 — un évènement, et un seul, par décision `Admis` ou `Refuse`.** `decider` émet, pour chaque société dont il
   rend `Admis` ou `Refuse`, **exactement un** évènement de cible `kesh_api::mcp::reseau`, de message fixe
   `serveur MCP : décision du filtre d'adresse`, avec ces champs, **dans cet ordre** (C-32-6c-2, C-32-6c-3) :

   | champ | valeur | forme |
   |---|---|---|
   | `company_id` | `s.company_id` | entier |
   | `filtre` | `s.address_filter_enabled` | booléen |
   | `plages` | `s.allowed_networks.len()` (entrées stockées, valides ou non) | entier |
   | `pair` | le pair TCP normalisé (`to_canonical`) | `%` (adresse) |
   | `pair_mandataire` | le pair appartient-il aux mandataires analysés de `s` | booléen |
   | `xff_lignes` | nombre de lignes `X-Forwarded-For` (0 : absent) | entier |
   | `xff_entrees` | nombre de morceaux séparés par des virgules, toutes lignes, comptés **sur les octets** (une ligne non ASCII compte ; une ligne vide compte 1) | entier |
   | `x_real_ip` | `X-Real-IP` présent | booléen |
   | `forwarded` | `Forwarded` présent | booléen |
   | `client` | le client que retient `reseau::client`, ou `aucun` | `%` |
   | `verdict` | `admis` ou `refuse` | texte |

   **Aucun autre champ.** Un seul site d'émission (une fonction de `reseau.rs` appelée par `decider`), deux niveaux (AC-2).
2. **AC-2 — les niveaux.** `Refuse` → **`WARN`** (visible au niveau par défaut, `info`) ; `Admis` → **`DEBUG`**
   (visible sous `RUST_LOG=…,kesh_api::mcp::reseau=debug`, AC-8). Les champs sont **les mêmes** aux deux niveaux
   (C-32-6c-4).
3. **AC-3 — le filtre désactivé journalise ce qu'il retiendrait.** `address_filter_enabled = FALSE` : la décision
   reste `Admis` dans tous les cas (fait 3 ; aucun test existant ne change), mais le client est **calculé** pour le
   journal — `client=aucun` si l'en-tête d'un mandataire déclaré est illisible — et l'évènement part en `DEBUG` avec
   `filtre=false`, `verdict=admis` (C-32-6c-5). C'est ce qui permet de vérifier le client vu **avant** d'activer le
   filtre.
4. **AC-4 — l'issue `Ecartee` n'a pas d'évènement de décision.** Une entrée invalide de `trusted_proxies` garde son
   `error!` actuel (`reseau.rs:110-114`, inchangé) et **n'émet pas** l'évènement de l'AC-1 : la décision a déjà le
   sien, et aucun client n'est calculé (le mandataire illisible pourrait être pris pour le client).
5. **AC-5 — rien que des valeurs dérivées.** L'évènement ne porte **aucune valeur d'en-tête** reçue (ni
   `X-Forwarded-For`, ni `X-Real-IP`, ni `Forwarded`, ni `Origin`, ni `Authorization`, ni `Cookie`), aucun corps, aucune
   clé, aucune donnée comptable : seulement des adresses **analysées** (`IpAddr`), des comptes, des booléens, l'id de
   société et deux mots fixes (C-32-6c-6). Garde : test `journal_du_filtre_sans_secret` (AC-T4).
6. **AC-6 — rien ne change pour le client.** Les réponses de `/mcp` sont **octet pour octet** celles de `c5287dec` :
   refusé = réponse de route absente, admis = suite des contrôles. Les tests (a) à (s) de `mcp_reseau_e2e.rs` et ceux de
   `mcp_transport_e2e.rs` passent **sans modification** ; les tests neufs comparent aussi la réponse à la route absente
   (`verdict(…)`, `mcp_reseau_e2e.rs:44-68`). Le journal ne devient pas un oracle : il ne sort pas du serveur.
7. **AC-7 — le serveur désactivé n'écrit rien.** Sans société `enabled = TRUE`, aucune décision n'est prise (fait 7) :
   **aucun** évènement de décision. Le manuel le dit (une absence de ligne veut dire « serveur MCP activé pour aucune
   société », ou « requête jamais parvenue à Kesh »).

### B — La directive de journal

8. **AC-8 — la directive ciblée n'ouvre que le filtre.** `build_log_filter("info,kesh_api::mcp::reseau=debug")` laisse
   passer un `debug!` de cible `kesh_api::mcp::reseau`, **pas** un `debug!` d'une autre cible de `kesh_api` ni un
   `info!` de `sqlx` (baseline `sqlx=warn` maintenue), et laisse passer le `warn!`. Test unitaire dans
   `logging.rs` (AC-T6) : la phrase du manuel est prouvée par le code qu'elle décrit, non par la documentation de la
   dépendance.

### C — Le manuel

9. **AC-9 — le manuel d'administration dit comment lire ce journal.** Une sous-section neuve
   `\subsection{Diagnostiquer le filtre d'adresse du serveur MCP}\label{sec:mcp-journal-filtre}`, dans § *Monitoring et
   logs*, après § *Logs applicatifs* (`admin-manual.tex:3019-3027`), « depuis la 0.16.0 », dit **tel que le code le
   fait** (C-32-6c-7) :
   - un refus s'écrit **toujours** (niveau `warn`) ; une admission seulement au niveau `debug`, à activer **pour la
     seule cible du filtre** : `RUST_LOG=info,kesh_api::mcp::reseau=debug` dans le `.env`, puis
     `docker compose up -d` (`sudo` sur DSM) pour recréer le conteneur ; **revenir à `info` ensuite** ; ne pas employer
     `RUST_LOG=debug` tout court (tout Kesh passe en `debug`) ;
   - lire : `docker compose logs --since=10m kesh-api | grep "filtre d'adresse"` (et le fichier de
     § `sec:logs-fichier`, s'il est actif) ;
   - **une ligne d'exemple copiée d'une sortie réelle** du binaire (fait 9), adresses de documentation ;
   - chaque champ en une ligne, et ce qu'il permet de trancher : un tableau de lecture au moins pour ces cas —
     (i) `pair` = une adresse de passerelle Docker, `pair_mandataire=false`, `xff_lignes=0` : le proxy ne pose pas
     `X-Forwarded-For`, Kesh voit tous les clients sous l'adresse du proxy, le filtre ne distingue personne ;
     (ii) même pair, `pair_mandataire=false`, `xff_lignes` ≥ 1 : l'en-tête est posé mais ignoré — déclarer le pair
     parmi les mandataires de confiance ; (iii) `pair_mandataire=true`, `client` = l'adresse du poste : le filtre voit
     le vrai client ; (iv) `client=aucun` : l'en-tête du mandataire porte une entrée qui n'est pas une adresse nue (port,
     `unknown`), la requête est refusée ; (v) `x_real_ip=true` ou `forwarded=true` : ces en-têtes ne sont **jamais**
     lus par `/mcp` ; (vi) `filtre=false` : la ligne dit ce que le filtre retiendrait, rien n'est refusé ; (vii)
     `filtre=true`, `plages=0` : personne n'est admis ; (viii) aucune ligne : serveur MCP activé pour aucune société,
     ou requête jamais parvenue à Kesh ; une entrée invalide de mandataire donne, à la place, une **erreur**
     `trusted_proxies` (AC-4) ;
   - avec plusieurs sociétés activées, **une ligne par société** (`company_id`) : un refus pour l'une n'empêche pas
     l'autre de servir ;
   - **ce que le journal ne contient jamais** : aucune clé, aucun en-tête `Authorization`, aucun corps, aucune valeur
     d'en-tête reçue (des comptes et des présences seulement), aucune donnée comptable ; il contient en revanche
     l'**adresse** des clients (comme le journal de connexion) ;
   - en cas de requêtes refusées en nombre, chacune écrit une ligne (aucune limite de débit sur `/mcp`).
10. **AC-10 — DSM renvoie au diagnostic.** À l'étape 4 de § `sec:synology` (`admin-manual.tex:652-655`), une phrase :
    le serveur MCP n'emploie **pas** `X-Real-IP` ; ce que le filtre voit derrière le proxy de DSM se vérifie au journal
    (\S\ref{sec:mcp-journal-filtre}). **Aucune prescription** d'en-tête supplémentaire à DSM tant qu'elle n'a pas été
    vérifiée (C-32-6c-9 ; question 4).
11. **AC-11 — PDF.** `admin-manual.pdf` régénéré (`make fr` dans `docs/manual/`, sous le verrou) et commité ; le PDF
    aplati (`pdftotext … - | tr '\n' ' ' | tr -s ' '`) contient « Diagnostiquer le filtre d'adresse », la directive
    `kesh_api::mcp::reseau=debug` et la phrase de l'AC-10. `user-manual` et brochure : inchangés. **CHANGELOG
    inchangé** (C-32-6c-10).

### T — Les tests

12. **AC-T1 — `decision_journalisee_par_cas`** : pour chaque cas, la requête, puis **la liste des évènements de
    décision** (évènements de la capture dont le texte contient `décision du filtre d'adresse`) **égale** à la liste
    attendue, construite par une fonction `attendu(niveau, champs…)` qui suit l'ordre de l'AC-1 — égalité de
    chaîne complète, non `contains` (C-32-6c-3). Cas :

    | # | réglages | pair | en-têtes | verdict, niveau | champs notables |
    |---|---|---|---|---|---|
    | 1 | défauts (filtre, 7 plages) | `198.51.100.7` | — | refuse, WARN | `pair_mandataire=false xff_lignes=0 client=198.51.100.7` |
    | 2 | défauts | `192.168.1.20` | — | admis, DEBUG | `client=192.168.1.20` |
    | 3 | mandataire `172.18.0.1` | `172.18.0.1` | XFF `198.51.100.7` | refuse, WARN | `pair_mandataire=true xff_lignes=1 xff_entrees=1 client=198.51.100.7` |
    | 4 | mandataire `172.18.0.1` | `172.18.0.1` | XFF `192.168.1.20`, `X-Real-IP: 198.51.100.7` | admis, DEBUG | `x_real_ip=true client=192.168.1.20` |
    | 5 | défauts (aucun mandataire) | `172.18.0.1` | XFF `198.51.100.7, 192.0.2.10`, `Forwarded: for=192.0.2.10` | admis, DEBUG | `pair_mandataire=false xff_lignes=1 xff_entrees=2 forwarded=true client=172.18.0.1` — **le cas DSM non déclaré** |
    | 6 | mandataire `172.18.0.1` | `172.18.0.1` | XFF `unknown` | refuse, WARN | `client=aucun` |
    | 7 | mandataire `172.18.0.1` | `172.18.0.1` | deux lignes XFF : `192.0.2.10, 192.168.1.20` puis `198.51.100.7` | refuse, WARN | `xff_lignes=2 xff_entrees=3 client=198.51.100.7` |
    | 8 | filtre désactivé | `198.51.100.7` | — | admis, DEBUG | `filtre=false client=198.51.100.7` |
    | 9 | filtre désactivé, mandataire `172.18.0.1` | `172.18.0.1` | XFF `unknown` | admis, DEBUG | `filtre=false client=aucun` |
    | 10 | filtre activé, `allowed_networks = []` | `192.168.1.20` | — | refuse, WARN | `plages=0` |
    | 11 | défauts | `2001:db8::7` | — | refuse, WARN | `pair=2001:db8::7 client=2001:db8::7` |
    | 12 | défauts | `::ffff:198.51.100.7` | — | refuse, WARN | `pair=198.51.100.7` (normalisé) |

    Chaque cas compare aussi la réponse à la route absente (`verdict(…)`) : refusé ⇔ route absente (AC-6). La
    capture s'installe **par cas** (après la pose des réglages), de sorte qu'une liste attendue d'**un** élément prouve
    « un seul évènement » (AC-1). `verdict(…)` envoie aussi la requête de route absente, qui ne passe pas par
    `decider` : elle n'ajoute aucun évènement (à vérifier à l'étape 0, sinon filtrer sur la seule requête `/mcp`).
13. **AC-T2 — `ecartee_sans_evenement_de_decision`** : `trusted_proxies = ["pas-un-mandataire"]` → la liste des
    évènements de décision est **vide** ; témoin : l'`error!` `trusted_proxies` est présent dans la même capture
    (`exiger_ensemble`, comme `mcp_reseau_e2e.rs:302-306`).
14. **AC-T3 — `serveur_desactive_aucun_evenement`** : aucune société activée → réponse de route absente, **aucun**
    évènement de décision ; **témoin dans la même capture** : la société activée ensuite, la même requête produit
    l'évènement attendu (sans quoi le test passerait à vide).
15. **AC-T4 — `journal_du_filtre_sans_secret`** : deux requêtes, l'une admise (pair `192.168.1.20`) et l'autre refusée
    (mandataire `172.18.0.1`, XFF piégé), portant `Authorization: Bearer kesh_mcp_PIEGE_JETON_326C`,
    `Cookie: kesh_session=PIEGE_COOKIE_326C`, `X-Forwarded-For: PIEGE_XFF_326C` (sur la requête refusée),
    `X-Real-IP: PIEGE_XRI_326C`, `Forwarded: for=PIEGE_FWD_326C`, `Origin: https://piege-origine-326c.example`, corps
    `{"PIEGE_CORPS_326C": …}` ; témoin : les deux évènements de décision existent ; puis `exiger_absent` pour chaque
    piège et pour `kesh_mcp_` — sur **tous** les évènements de la capture, non sur les seuls évènements de décision.
16. **AC-T5 — `deux_societes_deux_evenements`** : deux sociétés activées, l'une aux défauts, l'autre avec
    `allowed_networks = ["192.0.2.0/24"]` ; pair `192.0.2.10` → admis (par la seconde) ; la liste des évènements de
    décision compte **deux** éléments, `company_id` distincts, `refuse` en WARN pour la première, `admis` en DEBUG pour
    la seconde ; la comparaison se fait après tri par `company_id` (l'ordre de `list_enabled` n'est pas un contrat de
    cette story).
17. **AC-T6 — `filtre_cible_n_ouvre_que_le_filtre_mcp`** (`logging.rs`) : un abonné local (`with_default`) composé de
    `build_log_filter("info,kesh_api::mcp::reseau=debug")` et d'une couche de capture minimale ; évènements émis avec
    `target:` explicite — `debug!` de `kesh_api::mcp::reseau` (passe), `debug!` de `kesh_api::routes` (bloqué),
    `info!` de `sqlx::query` (bloqué), `warn!` de `kesh_api::mcp::reseau` (passe), `info!` de `kesh_api` (passe).

## Tâches

- [ ] **T-0 — étape 0** (§ *À recontrôler*), lignes lues citées au Change Log.
- [ ] **T-1 — l'évènement** (AC-1 à AC-5) : dans `mcp/reseau.rs`, une fonction privée `journaliser_decision(…)` (nom
  indicatif) appelée par `decider` à ses deux sorties `Admis`/`Refuse`, une seule liste de champs pour les deux niveaux
  (un `macro_rules!` local, ou deux appels dont l'égalité est tenue par AC-T1) ; `decider` calcule le client **avant**
  de consulter l'état du filtre, sans changer son résultat (AC-3) ; comptes d'en-têtes sur les octets ; mise à jour de la
  doc du module (`reseau.rs:1-23`), de celle de `decider` (`:101-103`) et de la ligne 2 de l'ordre des contrôles
  (`mcp/mod.rs:15`) : « un évènement par société décidée ».
- [ ] **T-2 — les tests** AC-T1 à AC-T5 dans `tests/mcp_reseau_e2e.rs` ; AC-T6 dans `logging.rs`.
- [ ] **T-3 — le manuel** (AC-9 à AC-11) : sous-section, phrase DSM, ligne d'exemple prise d'une sortie réelle (backend
  local de l'E2E, `curl` vers `/mcp` depuis `127.0.0.1` avec un réglage qui refuse, ou la capture du fichier de log),
  PDF régénéré sous le verrou ; contrôle du PDF aplati, sorties citées.
- [ ] **T-4 — mutations** (§ *Tests et mutations*), une à une, restauration par `git checkout --` + `touch`.
- [ ] **T-5 — gates** (§ *Gates*), Dev Agent Record, `sprint-status.yaml` → `review`, prompt de revue P1.

## Tests et mutations prévus

| # | mutation (dans `mcp/reseau.rs` sauf mention) | test qui doit rougir |
|---|---|---|
| M1 | évènement retiré à la sortie `Refuse` | AC-T1 (cas 1, 3, 6, 7, 10, 11, 12) |
| M2 | `Refuse` journalisé en `debug!` | AC-T1 (cas 1 : `WARN` attendu) |
| M3 | `Admis` journalisé en `info!` (ou `warn!`) | AC-T1 (cas 2 : `DEBUG` attendu) |
| M4 | retour anticipé du filtre désactivé gardé avant l'évènement (pas d'évènement) | AC-T1 (cas 8, 9) |
| M5 | `client` = le pair au lieu du client retenu | AC-T1 (cas 3, 4, 7) |
| M6 | `xff_entrees` compte les lignes | AC-T1 (cas 5 : 2 attendu ; cas 7 : 3 attendu) |
| M7 | `x_real_ip` toujours `false` | AC-T1 (cas 4) |
| M8 | `forwarded` lu sur `x-forwarded-for` (présence confondue) | AC-T1 (cas 3 : `forwarded=false` attendu, XFF présent) |
| M9 | `pair_mandataire` calculé contre `allowed_networks` | AC-T1 (cas 2 : `pair_mandataire=false` attendu, le pair étant dans les plages) |
| M10 | valeur brute de `X-Forwarded-For` ajoutée en champ | AC-T4 (`PIEGE_XFF_326C`) et AC-T1 (égalité) |
| M11 | `en_tetes = ?en_tetes` ajouté en champ | AC-T4 (`PIEGE_JETON_326C`, `PIEGE_COOKIE_326C`) |
| M12 | évènement émis aussi pour `Ecartee` (avant le contrôle des mandataires) | AC-T2 |
| M13 | évènement émis dans la boucle des plages (un par entrée) | AC-T1 (cas 1 : un élément attendu, sept émis) |
| M14 | évènement émis une seconde fois au transport (`transport.rs`, après le filtre) | AC-T1 (un élément attendu) |
| M15 | `decider` du filtre désactivé rend `Refuse` quand `client` est `None` (le calcul anticipé change le verdict) | AC-T1 cas 9 (`admis` attendu, et la réponse n'est plus celle d'un admis) |
| M16 | `build_log_filter` remplace la chaîne par `debug` quand elle contient `::` (directive ciblée élargie) | AC-T6 (`kesh_api::routes` bloqué) |

⚠️ **M9 et le choix des adresses** : `172.18.0.1` est dans `172.16.0.0/12` (plages par défaut, `tests/common/mcp.rs:162`),
si bien que, au cas 3, mandataire et plage donnent tous deux `true` ; c'est le **cas 2** (pair `192.168.1.20` dans les
plages, hors des mandataires) qui tue M9. Si l'étape 0 veut une mutation tuée par deux cas, ajouter un cas où le
mandataire est hors des plages (`192.0.2.1` déclaré, réglages par défaut). M15 : la mutation la plus probable du
développement (le calcul du client avancé) ; le cas 9 la tue seul, le test (d) ne voit que le pair public sans en-tête.

Tests existants qui doivent rester verts **sans modification** : tout `mcp_reseau_e2e.rs` (a) à (s), (n'),
`mcp_transport_e2e.rs` (dont `jeton_jamais_journalise`, qui voit désormais les évènements de décision et y cherche le
jeton), `mcp_journal_e2e.rs`, les tests unitaires de `reseau.rs` et de `logging.rs`.

## Gates

- Entre les passes : gate ciblé — `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo nextest run -E 'binary(mcp_reseau_e2e) | binary(mcp_transport_e2e) | binary(mcp_journal_e2e)'` et les tests de
  bibliothèque de `kesh-api` (`logging.rs`, `mcp/reseau.rs`).
- **Au dernier commit de code** : base remise à zéro (bases propres au worktree, **sans** redémarrer le conteneur,
  `docs/agents-orchestration.md` § 2), **gate complet** `scripts/test-fast.sh --ci` sous `flock …/.gate.lock` ; frontend
  `check`, `lint-i18n-ownership`, `test:unit`, `build` (aucun fichier du frontend ne change : le build sert l'E2E) ;
  **E2E complet** (`docs/testing.md` § *Les échecs attendus*). Aucune migration : pas d'exception `kesh-db`.
- **Recette visuelle : sans objet** — aucun contrôle visible n'est ajouté ni modifié.
- Revue de code : P1 complète (Sonnet ou Opus, jamais Haiku), lentilles B, E, A ; axes nommés : **le manuel contre le
  code et le PDF aplati**, **aucune valeur d'en-tête au journal**, **réponses inchangées**.

## Ce que les gardes ne voient pas

- **Ce que DSM fait réellement.** Les tests simulent un mandataire ; seul le NAS dira quel pair Kesh voit derrière le
  proxy de DSM, et si DSM pose `X-Forwarded-For`. C'est l'objet même de la story : le journal le dira à la mise à jour.
- **Un second site d'émission futur.** AC-T1 compare la liste complète des évènements de décision sur douze cas : un
  champ ajouté à **ce** site rougit ; un **autre** évènement, écrit ailleurs dans `mcp/` avec une valeur d'en-tête, ne
  rougit que si un test d'absence (AC-T4, `jeton_jamais_journalise`) passe par lui.
- **La forme d'une entrée illisible.** Le journal compte les entrées et dit `client=aucun`, sans la valeur (C-32-6c-6) :
  l'administrateur sait **qu'**une entrée n'est pas une adresse nue, non **laquelle** (port, `unknown`, crochets). Et
  tant que le pair n'est pas déclaré mandataire, l'en-tête n'est pas analysé : `xff_entrees` en donne le nombre, non la
  lisibilité.
- **L'amplification au niveau par défaut.** Chaque requête refusée écrit un `warn!` par société activée, sans limite
  de débit (décision de Guy : pas de limite en 0.16.0). Même classe que le `warn!` par requête de la connexion limitée
  (`routes/auth.rs:190`) et que l'`error!` d'une entrée de mandataire invalide (fiche 32-6, § *Ce que les gardes ne
  voient pas*, « amplification du journal du serveur »). Le fichier de log tourne chaque jour, sans borne de taille
  (§ `sec:logs-fichier`). Question 1.
- **Plusieurs sociétés** : un `warn` de refus pour une société n'empêche pas une autre de servir le même client ; le
  manuel le dit, le journal ne résume pas la requête.
- **L'adresse des clients** est une donnée personnelle de la personne qui se connecte ; elle est déjà au journal de
  connexion (fait 11), et le fichier de log est dans le périmètre de la sauvegarde de l'installation (§
  `sec:logs-fichier`).
- **Le temps de réponse** : l'évènement ajoute une mise en forme par décision, non mesurée ; l'oracle temporel de la
  fiche 32-6 (la décision lit la base, une route absente non) reste ce qu'il était.
- **La mise en forme réelle** (guillemets, ordre, format `json`) n'est vue d'aucun test : l'exemple du manuel se copie
  d'une sortie réelle (fait 9), et la revue le relit.
- **Le manuel** : aucun gate ne le lit ; le prompt de revue nomme l'axe.

## Dev Notes

### Esquisse (indicative dans la forme, exigée dans le fond)

```rust
// mcp/reseau.rs — `decider`, après la lecture des mandataires (inchangée, `Ecartee` sans évènement)
let pair = pair.to_canonical();
let retenu = client(pair, en_tetes, &mandataires);          // calculé même filtre désactivé (AC-3)
let decision = if !s.address_filter_enabled {
    Decision::Admis
} else if let Some(c) = retenu {
    if admis_par_les_plages(s, c) { Decision::Admis } else { Decision::Refuse }
} else {
    Decision::Refuse
};
journaliser_decision(s, pair, contient(&mandataires, pair), en_tetes, retenu, decision);
decision
```

- Les comptes : `en_tetes.get_all("x-forwarded-for").iter()` — lignes ; morceaux par
  `v.as_bytes().split(|b| *b == b',').count()` sommés ; présence par `en_tetes.contains_key("x-real-ip")` et
  `"forwarded"`. **Aucune** de ces valeurs n'est convertie en texte.
- `client` : un petit type `Display` (« adresse » ou `aucun`) passé par `%`, pour une ligne sans guillemets.
- La boucle des plages et son `debug!` d'entrée ignorée (`reseau.rs:126-134`) restent ; seule leur place change si le
  calcul est réordonné. Le `debug!` d'entrée ignorée **n'est pas** un évènement de décision (filtrage des tests sur le
  message).

### Fichiers touchés (prévus) et modules

- `crates/kesh-api/src/mcp/reseau.rs`, `crates/kesh-api/src/mcp/mod.rs` (doc) — module **`mcp`** ;
  `crates/kesh-api/tests/mcp_reseau_e2e.rs` — rattaché au module `mcp` (convention de la fiche 32-6, `tests/mcp_*`) ;
- `crates/kesh-api/src/logging.rs` (`mod tests` seulement) — module **`logging`** ;
- `docs/manual/fr/admin-manual.tex` et `.pdf` — **manuel**.

**Trois modules** (≤ 5). `main.rs`, la configuration, les composes et `.env.example` ne changent pas : la directive
passe par `RUST_LOG`, déjà transmis (fait 8 ; C-32-6c-7).

### Le manuel (axe de revue)

- Vérifier l'AC-9 et l'AC-10 contre le code livré et contre le **PDF aplati** ; `grep -n "X-Real-IP\|RUST_LOG\|/mcp"
  docs/manual/fr/admin-manual.tex` avant et après ; aucune autre phrase du manuel ne devient fausse (la Traefik,
  § `sec:traefik`, pose ses propres en-têtes : la sous-section en parle-t-elle juste ? Elle ne prescrit rien pour
  Traefik ; la 32-7 écrit la section *Le serveur MCP* et y renverra).
- **Pour la 32-7** (à reporter par l'orchestrateur dans sa fiche) : la section *Le serveur MCP* renvoie à
  \S\ref{sec:mcp-journal-filtre} pour le diagnostic ; et un point que cette fiche n'écrit pas au manuel : déclarer la
  passerelle Docker comme mandataire fait lire `X-Forwarded-For` à **toute** requête qui passe par elle, y compris
  celle d'un poste du réseau local qui joindrait le port publié directement, sans DSM, et qui pourrait alors forger
  l'en-tête — à dire avec ce que le filtre ne garantit pas.

### Références

- Fiche 32-6 : AC-a7, AC-a8, AC-a9, AC-b4 ; § *Ce que les gardes ne voient pas* ; § *Le manuel (axe de revue)* ;
  points 2 et 2 bis pour Guy. Registre 326 : C-32-6-4, C-32-6-5, C-32-6-6.
- Epic 32, § 32-6c ; § *Réponses de Guy (2026-10-25)* : pas de limite de débit en 0.16.0, le manuel dit ce que le
  filtre ne garantit pas.

## Points à soumettre à Guy

1. **Un refus du filtre au niveau `warn`, donc visible sans rien régler — d'accord ?** Chaque requête refusée écrit
   une ligne, sans limite de débit ; un robot qui frappe `/mcp` depuis l'extérieur remplirait le journal comme il le
   remplit déjà en frappant la page de connexion (`warn` par requête une fois la limite atteinte). *Ma recommandation :
   `warn`* — c'est le cas qui demande une action, et un refus d'un poste légitime doit se voir sans changer `RUST_LOG`.
   L'autre choix : tout en `debug`, le journal ne disant rien tant qu'on ne l'a pas demandé.
2. **Ne journaliser que la forme des en-têtes (nombre de lignes et d'entrées, présence), jamais leur valeur ?**
   *Ma recommandation : oui.* La valeur d'un en-tête est écrite par le client, donc par qui veut : la recopier ouvrirait
   le journal à des textes forgés. Le client **retenu**, lui, est une adresse analysée, et il suffit au diagnostic. Le
   coût : si une entrée est illisible, le journal dit `client=aucun` sans dire laquelle.
3. **Le filtre désactivé journalise-t-il quand même le client qu'il retiendrait ?** *Ma recommandation : oui* (en
   `debug`) : c'est ce qui te permet, à la mise à jour, de vérifier ce que Kesh voit derrière DSM **avant** d'activer le
   filtre, sans risquer de couper ton agent.
4. **Faut-il dès maintenant conseiller au manuel d'ajouter `X-Forwarded-For` aux en-têtes personnalisés de DSM ?**
   *Ma recommandation : non.* Ni la variable que DSM accepte ni son effet ne sont vérifiés ; la fiche fait dire au
   manuel que `/mcp` ignore `X-Real-IP` et où lire le diagnostic. Ce que la mise à jour du NAS établira, la 32-7 (ou un
   correctif du manuel) l'écrira.
5. **Le paragraphe du manuel avant la section *Le serveur MCP*** : la 32-6c l'écrit sous *Monitoring et logs*, la 32-7
   y renverra. *Ma recommandation : oui* — un diagnostic se cherche avec les journaux ; l'autre choix (attendre la
   32-7) laisserait la story sans son mode d'emploi.

## À recontrôler à l'étape 0 du développement

- `reseau.rs:63-87` (`client`), `:104-140` (`decider`), `:110-114` (`error!`), `:129-132` (`debug!`) ; `transport.rs:81`,
  `:84-97` ; `mcp/mod.rs:15` — aucune story fusionnée entre-temps (32-8b, 32-10a) ne les a-t-elle changés ?
  `git log c5287dec..origin/main -- crates/kesh-api/src/mcp/reseau.rs crates/kesh-api/src/mcp/transport.rs`.
- `tests/common/capture_trace.rs:30-47` (format ` nom=valeur`, sans guillemets, `record_str`) et
  `tests/common/mcp.rs` (`reglages` `:137`, `RESEAUX_DEFAUT` `:162`, `activer` `:165`, `livres` `:171`, `societe`
  `:64`) ; `verdict(…)` (`mcp_reseau_e2e.rs:44-68`) : la requête de route absente n'émet-elle aucun évènement de
  `reseau` ? (sinon filtrer).
- `logging.rs:104-120`, `:129-131` ; `tracing-subscriber` toujours en `0.3.23` au `Cargo.lock` (faits 8 et 9).
- Le manuel : `admin-manual.tex:638-661`, `:696`, `:3017-3027` ; la 32-7 a-t-elle écrit la section *Le serveur MCP*
  entre-temps ? Si oui, le renvoi de l'AC-10 s'ajoute aussi là, et la sous-section de l'AC-9 garde sa place.
- L'ordre des champs de l'AC-1 se retrouve-t-il dans la sortie réelle (fait 9) ? Le copier tel quel dans la ligne
  d'exemple du manuel.

## Dev Agent Record

### Agent Model Used

*(à remplir)*

### Completion Notes List

*(à remplir)*

### File List

*(à remplir)*

## Change Log

| Date | Étape | Résumé |
|---|---|---|
| 2026-10-25 | création | Fiche rédigée en autonomie (Opus 5.5) contre `c5287dec`, sur la demande de Guy du même jour (« ajoute le nécessaire dans les logs au cas où ») ; registre C-32-6c-1 à C-32-6c-10 ; epic annoté (§ 32-6c, après la 32-6) ; `sprint-status.yaml` : `32-6c-journal-filtre-adresse: ready-for-dev` ; prompt de validation P1 `326c-validate-prompt-p1.md` (Sonnet, lentilles R et F). Constat au relevé : C-32-6-4 et l'AC-b4 promettaient le refus « au journal du serveur », que le code n'écrit pas (fait 2). Décomptes (recomptés sur la fiche) : 17 AC (11 + 6 tests), 6 tâches (T-0 à T-5), 16 mutations (M1 à M16), 12 cas à l'AC-T1, 3 modules. |

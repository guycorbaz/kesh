UPDATE audit_log a
    LEFT JOIN users u ON u.id = a.user_id
   SET a.actor_label = COALESCE(u.username, '(inconnu)')
 WHERE a.actor_label = '';

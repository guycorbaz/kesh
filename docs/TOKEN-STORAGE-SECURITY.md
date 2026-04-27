# Token Storage Security Guide

**Document Type:** Security Best Practices  
**Status:** Recommended for v0.1+ implementation  
**Last Updated:** 2026-04-25

---

## Current State (v0.1)

### Access Token Storage
- **Location:** Memory (SvelteKit session store)
- **Persistence:** Cleared on page reload (intentional)
- **Vulnerability:** N/A (memory-only)

### Refresh Token Storage
- **Location:** localStorage
- **Persistence:** Survives page reload
- **Vulnerability:** ⚠️ Vulnerable to XSS attacks
  - If XSS payload executes, attacker can read localStorage.refreshToken
  - Attacker can refresh access token and impersonate user

---

## Recommended Migration: httpOnly Cookies

### Why httpOnly Cookies?

1. **JavaScript cannot access** — XSS payloads cannot read the cookie
2. **Automatic transmission** — Browsers automatically include in requests
3. **Secure flag** — Can enforce HTTPS-only transmission
4. **SameSite** — Can prevent CSRF attacks

### Implementation Strategy

#### Phase 1: Backend Changes (Story X-Y)
1. **Modify login endpoint** to set httpOnly cookies:
   ```rust
   // Instead of returning tokens in JSON:
   // LoginResponse { access_token, refresh_token }
   
   // Return cookies:
   let mut response = Json(LoginResponse { 
       expires_in: 900  // Only send expiry, not actual tokens
   }).into_response();
   
   response.headers_mut().insert(
       SET_COOKIE,
       HeaderValue::from_str(&format!(
           "accessToken={}; Path=/api; HttpOnly; Secure; SameSite=Strict; Max-Age=900",
           access_token
       ))?
   );
   
   response.headers_mut().insert(
       SET_COOKIE,
       HeaderValue::from_str(&format!(
           "refreshToken={}; Path=/api/auth/refresh; HttpOnly; Secure; SameSite=Strict; Max-Age=2592000",
           refresh_token
       ))?
   );
   ```

2. **Modify refresh endpoint** to accept cookie-based tokens
3. **Modify logout endpoint** to clear cookies

#### Phase 2: Frontend Changes (Story X-Y+1)
1. **Remove localStorage token handling**
2. **Rely on automatic cookie transmission**
3. **Verify credentials via GET /api/v1/auth/me** (new endpoint)
4. **Handle cookie errors gracefully** (redirect to login on 401)

#### Phase 3: Deployment
1. **Verify Reverse Proxy** supports httpOnly cookies
   - nginx: `proxy_cookie_flags ~* "^.*" httponly secure samesite=strict;`
   - Caddy: Built-in support
2. **Enforce HTTPS** in production
3. **Monitor:** Log any cookie-related errors

---

## Backward Compatibility

**v0.1:** JSON token response (current, localStorage)  
**v0.2:** Deprecate JSON token response, use httpOnly cookies  
**v0.3+:** Remove JSON token response entirely

---

## Testing Checklist

- [ ] XSS mitigation: Verify JavaScript cannot access httpOnly cookies
- [ ] CSRF mitigation: Verify SameSite=Strict blocks cross-site requests
- [ ] Automatic transmission: Verify refresh works without explicit Cookie header
- [ ] Logout: Verify cookies are cleared
- [ ] Session persistence: Verify tokens survive page reload
- [ ] Secure flag: Verify cookies only transmitted over HTTPS

---

## References

- [OWASP: Session Management](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
- [MDN: httpOnly Cookies](https://developer.mozilla.org/en-US/docs/Web/HTTP/Cookies#security)
- [Axum: Setting Cookies](https://docs.rs/axum/latest/axum/http/header/index.html)

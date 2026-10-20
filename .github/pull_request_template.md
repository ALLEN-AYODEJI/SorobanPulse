## Summary

<!-- A clear, concise description of what this PR does. -->

## Related Issue

<!--
  One issue per PR. Only list several issues if they are tightly related,
  and explain why below. You must be assigned to every issue you close.
  See CONTRIBUTING.md → "Pull Requests".
-->

Closes #<!-- issue number -->

- [ ] I am assigned to the linked issue(s)
- [ ] This PR addresses a single issue (or tightly related issues, explained in Notes)

## Changes

<!-- List the key changes made. -->

-

## Testing

<!-- Describe how you tested this. -->

- [ ] `make test` passes
- [ ] `make lint` (fmt + clippy) reports no warnings
- [ ] Required CI checks are green (see CONTRIBUTING.md → "Pull Requests")
- [ ] Manually tested locally

## Security Considerations

<!--
  Complete this section if your change crosses a trust boundary.
  Trust boundaries include: clients → API, API → DB, API → RPC,
  API → outbound integrations (webhooks, Discord, Slack, email, SMS),
  API → admin endpoints, or any change to auth/authz logic.

  If this PR does NOT cross a trust boundary, replace the checklist
  with "N/A — no trust boundary crossed."
-->

- [ ] All user-supplied URLs validated by the shared SSRF guard (`net::outbound::validate_outbound_url`)
- [ ] New outbound HTTP clients created via `net::outbound::build_outbound_client()` (redirects disabled, private-range resolver)
- [ ] New unauthenticated endpoints protected with per-IP rate limiting (separate from the main API limiter)
- [ ] Tokens embedded in emails / webhooks are HMAC-signed with an expiry and verified in constant time
- [ ] Bounce/webhook endpoints authenticate the provider signature or shared secret before writing to the DB
- [ ] No secrets, PII, or credential material committed — `make security` scan passes
- [ ] `cargo deny check all` passes (no new advisory hits or disallowed licences)
- [ ] Dependency Review CI check passes (npm audit / pip-audit / govulncheck green)

## Notes

<!-- Anything reviewers should pay special attention to, or N/A. -->

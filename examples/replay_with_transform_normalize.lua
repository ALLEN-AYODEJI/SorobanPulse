-- Example transformation body for POST /v1/replay/with-transform
--
-- Unlike EVENT_TRANSFORM_SCRIPT files and /v1/admin/lua/preview, the
-- replay-with-transform endpoint wraps whatever you send in
-- `transformation_script` as the body of an implicit
-- `function transform_event(event) ... end` — so this file is the function
-- *body* only, not a standalone script defining the function itself.
--
-- Usage: send this file's contents (or an inline excerpt) as the
-- `transformation_script` field of a POST /v1/replay/with-transform request.

-- Drop diagnostic events entirely.
if event.event_type == "diagnostic" then
    return nil
end

-- Add a human-readable XLM amount alongside the raw stroop value.
if event.value and event.value.amount then
    event.value.amount_xlm = event.value.amount / 10000000
end

-- Tag the event so it's obvious downstream that this row went through a
-- replay pass rather than the original indexer.
if event.value then
    event.value.replayed = true
end

return event

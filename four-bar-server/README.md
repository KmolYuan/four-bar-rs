# Four🍀bar

The server of `four-bar-ui` client-side program.

## OAuth2 login with Google

Create credentials at <https://console.developers.google.com/apis/credentials>.

Goto "OAuth consent screen" create an APP. During development mode, only the test users can access.

Then, goto "Credentials" page add a "OAuth 2.0 Client IDs", use the "Client ID" and the "Client Secret" with this program.

Be aware that "Authorized redirect URIs" must add `http://localhost:PORT/auth` to allow redirect after login.

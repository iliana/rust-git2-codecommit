# rust-git2-codecommit

This is a credential provider that you can give to `git2::RemoteCallbacks::credentials` and have it [use AWS credentials from the usual locations](https://github.com/rusoto/rusoto/blob/master/AWS-CREDENTIALS.md).

It will then generate the username and password to use AWS CodeCommit via HTTPS, similar to the [AWS CLI credential helper](http://docs.aws.amazon.com/codecommit/latest/userguide/setting-up-https-unixes.html).

(You can also probably just set up the credential helper and libgit2 should do the right thing, but.)

This module uses code copied from private functions in [rusoto](https://github.com/rusoto/rusoto), available under the MIT license.

## Example

```rust
use git2::{FetchOptions, RemoteCallbacks};
use git2::build::RepoBuilder;
use git2_codecommit::codecommit_credentials;

let mut remote_cbs = RemoteCallbacks::new();
remote_cbs.credentials(codecommit_credentials);
let mut fetch_opts = FetchOptions::new();
fetch_opts.remote_callbacks(remote_cbs);
let repo = RepoBuilder::new().fetch_options(fetch_opts).clone(url, some_path).unwrap();
// etc.
```

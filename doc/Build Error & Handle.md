##### Build Error & Handle

🔥**warning**🔥

- For most node build errors , just check your rust version to the lastest.
- Blow, lists some build errors our dev team have met, just for reference.
- For more build errors, please refer the [substrate repo issue](https://github.com/paritytech/substrate/issues).



| No   | Error info                                                   | Handle in Linux                                              | Handle in Mac                                                |
| ---- | ------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ |
| 1    | linker `cc` not found                                        | **Reason**:clang env err<br />**Solution**:apt install clang | **Reason**:clang env err<br />**Solution**:brew install clang |
| 2    | failed to run custom build command for `librocksdb-sys v6.17.3` | **Reason**:clang env err<br />**Solution**:apt install clang | **Reason**:clang env err<br />**Solution**:brew install clang |
| 3    | linking with `cc` failed: exit code: 1<br/> symbol(s) not found for architecture x86_64 | **Reason**: DB file err when node update<br />**Solution**: ./target/release/dnft purge-chain --dev | **Reason**: DB file err when node update<br />**Solution**: ./target/release/dnft purge-chain --dev |
| 4    | memory link                                                  | **Reason**:rust thread err <br />**Solution**:cargo update -p thread_local | **Reason**:rust thread err<br />**Solution**:cargo update -p thread_local |


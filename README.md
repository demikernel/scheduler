Catwalk
==========

_Catwalk_ is a fast engine for scheduling futures.

> This project is a component of _Demikernel_ - a libOS architecture for
kernel-bypass devices.

> To read more about _Demikernel_ check out https://aka.ms/demikernel.

This project was inspired by many others. Special thanks to the following ones:

- [unicycle](https://github.com/udoprog/unicycle): a catwalk for driving a large number of futures.

Building and Running
---------------------

**1. Clone This Repository**
```
export WORKDIR=$HOME                                   # Change this to whatever you want.
cd $WORKDIR                                            # Switch to working directory.
git clone https://github.com/demikernel/catwalk.git    # Clone.
```

**1. Install Prerequisites**
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**3. Build the Sources**
```
cargo build
```

**4. Run Regression Tests**
```
cargo test
```

Code of Conduct
---------------

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/).
For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/)
or contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.


Usage Statement
--------------

This project is a prototype. As such, we provide no guarantees that it will
work and you are assuming any risks with using the code. We welcome comments
and feedback. Please send any questions or comments to one of the following
maintainers of the project:

- [Irene Zhang](https://github.com/iyzhang) - [irene.zhang@microsoft.com](mailto:irene.zhang@microsoft.com)
- [Pedro Henrique Penna](https://github.com/ppenna) - [ppenna@microsoft.com](mailto:ppenna@microsoft.com)

> By sending feedback, you are consenting that it may be used  in the further
> development of this project.

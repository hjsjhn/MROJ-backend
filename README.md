# 大作业：在线评测系统

2022 年夏季学期《程序设计训练》 Rust 课堂大作业（二）。

## 作业要求

具体要求请查看[作业文档](https://lab.cs.tsinghua.edu.cn/rust/projects/oj/)。

## Honor Code

请在 `HONOR-CODE.md` 中填入你完成作业时参考的内容，包括：

* 开源代码仓库（直接使用 `crate` 除外）
* 查阅的博客、教程、问答网站的网页链接
* 与同学进行的交流

## 自动测试

本作业的基础要求和部分提高要求可使用 Cargo 进行自动化测试。运行 `cargo test --test basic_requirements -- --test-threads=1` 可测试基础要求，`cargo test --test advanced_requirements -- --test-threads=1` 可测试部分提高要求。

如果某个测试点运行失败，将会打印 `case [name] incorrect` 的提示（可能会有额外的 `timeout` 提示，可以忽略）。你可以使用 `cargo test test_name` 单独运行此测试，也可以在 `tests/cases` 目录下查看相应测试用例的内容，并按照文档的说明调试。

自动测试运行每个测试点后，会生成以下的文件：

* `[case_name].stdout/stderr`：OJ 程序的标准输出和标准错误。你可以在代码中添加打印语句，然后结合输出内容来调试代码。
* `[case_name].http`：测试过程中发送的 HTTP 请求和收到的响应。调试时，你可以先自己启动一个 OJ 服务端（`cargo run`），然后用 VSCode REST Client 来手动发送这些 HTTP 请求，并观察响应。

项目配置了持续集成（CI）用于帮助你测试。在推送你的改动后，可以在 GitLab 网页上查看 CI 结果和日志。同时，上述的文件也会被收集到对应任务的 artifacts 中，你可以在 GitLab 网页上下载并查看。

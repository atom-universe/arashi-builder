import React from "react";
// import 语句会让浏览器把后面的内容当作是 esm 来请求
// 也就是会自动 res.set_content_type("application/javascript");
// 所以这里的 css 就被当成了 js 模块来请求， 不额外处理的话会报错
import "./Comp.css";

const Comp = () => {
  return (
    <div className="container">
      <h1>Arashi</h1>
      <img src="pic.png" alt="pic!" />
    </div>
  );
};

export default Comp;

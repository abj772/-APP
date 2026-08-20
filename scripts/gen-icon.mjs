// 生成"记记账"应用图标：绿色圆角渐变底 + 白色"记"字
import sharp from "sharp";
import path from "path";

const svg = `<svg width="1024" height="1024" viewBox="0 0 1024 1024" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#4CAF50"/>
      <stop offset="1" stop-color="#66BB6A"/>
    </linearGradient>
  </defs>
  <rect x="40" y="40" width="944" height="944" rx="200" fill="url(#g)"/>
  <text x="512" y="530" font-size="540" font-weight="bold" fill="#ffffff"
        font-family="'Microsoft YaHei', 'PingFang SC', 'SimHei', sans-serif"
        text-anchor="middle" dominant-baseline="central">记</text>
</svg>`;

const out = path.resolve("app-icon.png");
await sharp(Buffer.from(svg)).png().toFile(out);
console.log("图标已生成: " + out);

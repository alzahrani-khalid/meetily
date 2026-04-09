import { dirname } from "path";
import { fileURLToPath } from "url";
import { FlatCompat } from "@eslint/eslintrc";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const compat = new FlatCompat({
  baseDirectory: __dirname,
});

const physicalDirectionMessage =
  "Use logical Tailwind properties (ms-/me-/ps-/pe-/text-start/text-end/border-s-/border-e-/rounded-s-/rounded-e-) for RTL support.";

const eslintConfig = [
  ...compat.extends("next/core-web-vitals", "next/typescript"),
  {
    files: ["src/**/*.tsx"],
    rules: {
      "no-restricted-syntax": [
        "error",
        {
          selector:
            "JSXAttribute[name.name='className'] Literal[value=/\\b(m[lr]-|p[lr]-|text-left|text-right|border-[lr]-|rounded-[lr]-)/]",
          message: physicalDirectionMessage,
        },
        {
          selector:
            "JSXAttribute[name.name='className'] TemplateLiteral TemplateElement[value.raw=/\\b(m[lr]-|p[lr]-|text-left|text-right|border-[lr]-|rounded-[lr]-)/]",
          message: physicalDirectionMessage,
        },
        {
          selector:
            "CallExpression[callee.name='cn'] Literal[value=/\\b(m[lr]-|p[lr]-|text-left|text-right|border-[lr]-|rounded-[lr]-)/]",
          message: physicalDirectionMessage,
        },
        {
          selector:
            "CallExpression[callee.name='clsx'] Literal[value=/\\b(m[lr]-|p[lr]-|text-left|text-right|border-[lr]-|rounded-[lr]-)/]",
          message: physicalDirectionMessage,
        },
      ],
    },
  },
];

export default eslintConfig;

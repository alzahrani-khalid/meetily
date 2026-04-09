"use client";

/**
 * BlockNote RTL Spike Test Page
 *
 * Purpose: Time-boxed investigation answering 4 questions from spec section 7
 * about BlockNote v0.36.0's RTL capabilities. Kept in repo as permanent evidence
 * per decision D-03.
 *
 * Questions tested:
 *   Q1: Does BlockNote render RTL text correctly?
 *   Q2: Does the slash menu work in RTL?
 *   Q3: Can the dictionary prop localize block labels?
 *   Q4: Is cursor behavior correct in RTL?
 *
 * Requirement: UI-05
 */

import { PartialBlock } from "@blocknote/core";
import "@blocknote/shadcn/style.css";
// NOTE: Do NOT import @blocknote/core/fonts/inter.css
// Inter lacks Arabic glyphs (RESEARCH Pitfall 3). The editor inherits
// Tajawal (RTL) or Source Sans 3 (LTR) from the global CSS cascade.
import { ar } from "@blocknote/core/locales";

// Multi-line Arabic initial content for testing all 4 spike questions
const initialContent: PartialBlock[] = [
  {
    type: "heading",
    content: "اختبار دعم اللغة العربية",
  },
  {
    type: "paragraph",
    content:
      "هذا نص تجريبي لاختبار عرض النص العربي من اليمين إلى اليسار في محرر BlockNote.",
  },
  {
    type: "paragraph",
    content:
      "اكتب / لفتح قائمة الأوامر وتحقق من ظهور التسميات باللغة العربية.",
  },
  {
    type: "bulletListItem",
    content: "عنصر قائمة أول",
  },
  {
    type: "bulletListItem",
    content: "عنصر قائمة ثاني",
  },
  {
    type: "paragraph",
    content:
      "استخدم مفاتيح الأسهم للتنقل بين الأسطر واختبار سلوك المؤشر.",
  },
];

export default function BlockNoteRTLSpike() {
  // Lazy import to avoid SSR issues (same pattern as Editor.tsx)
  const { useCreateBlockNote } = require("@blocknote/react");
  const { BlockNoteView } = require("@blocknote/shadcn");

  const editor = useCreateBlockNote({
    dictionary: ar,
    initialContent: initialContent as PartialBlock[] | undefined,
  });

  return (
    <div
      dir="rtl"
      style={{ fontFamily: "var(--font-sans-ar)", padding: "2rem" }}
    >
      <h2 style={{ marginBottom: "1rem" }}>BlockNote RTL Spike Test</h2>

      <div
        style={{
          marginBottom: "1.5rem",
          padding: "1rem",
          border: "1px solid #e5e7eb",
          borderRadius: "0.5rem",
          fontSize: "0.875rem",
          color: "#6b7280",
        }}
      >
        <p style={{ fontWeight: 600, marginBottom: "0.5rem" }}>
          Test Instructions:
        </p>
        <ul style={{ listStyleType: "disc", paddingInlineStart: "1.5rem" }}>
          <li>
            <strong>Q1 (RTL text rendering):</strong> Verify Arabic text is
            right-aligned with correct line wrapping
          </li>
          <li>
            <strong>Q2 (Slash menu):</strong> Type / anywhere in the editor and
            check that the menu opens with correct RTL positioning
          </li>
          <li>
            <strong>Q3 (Dictionary prop):</strong> Check that slash menu items
            and placeholders appear in Arabic (e.g. paragraph, heading labels)
          </li>
          <li>
            <strong>Q4 (Cursor behavior):</strong> Use arrow keys to navigate
            between lines and verify correct RTL cursor movement
          </li>
        </ul>
      </div>

      <BlockNoteView editor={editor} editable={true} theme="light" />
    </div>
  );
}

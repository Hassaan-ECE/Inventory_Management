import { ChevronDownIcon, FileCodeIcon, FileSpreadsheetIcon, UploadIcon } from "lucide-react";

import {
  DropdownItem,
  DropdownPanel,
} from "@/shared/components/ui/DropdownMenu";
import { useDropdownMenu } from "@/shared/hooks/useDropdownMenu";
import { Button } from "@/shared/components/ui/button";

interface ExportMenuProps {
  onExportExcel: () => void;
  onExportHtml: () => void;
  onOpenChange?: (open: boolean) => void;
}

export function ExportMenu({ onExportExcel, onExportHtml, onOpenChange }: ExportMenuProps) {
  const { open, menuRef, toggle, close } = useDropdownMenu({ onOpenChange });

  return (
    <div className="relative" ref={menuRef}>
      <Button aria-expanded={open} aria-haspopup="menu" size="sm" variant="outline" onClick={toggle}>
        <UploadIcon className="size-3.5" />
        Export
        <ChevronDownIcon className="size-3.5" />
      </Button>
      {open ? (
        <DropdownPanel align="right" className="w-44" maxHeightClassName="max-h-none" title="Export">
          <DropdownItem
            onClick={() => {
              close();
              onExportExcel();
            }}
          >
            <FileSpreadsheetIcon className="size-4" />
            Excel
          </DropdownItem>
          <DropdownItem
            onClick={() => {
              close();
              onExportHtml();
            }}
          >
            <FileCodeIcon className="size-4" />
            HTML
          </DropdownItem>
        </DropdownPanel>
      ) : null}
    </div>
  );
}

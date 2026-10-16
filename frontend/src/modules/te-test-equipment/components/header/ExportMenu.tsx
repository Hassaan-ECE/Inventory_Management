import { FileSpreadsheetIcon, UploadIcon } from "lucide-react";

import {
  DropdownItem,
  DropdownPanel,
} from "@/shared/components/ui/DropdownMenu";
import { useDropdownMenu } from "@/shared/hooks/useDropdownMenu";
import { Button } from "@/shared/components/ui/button";

interface ExportMenuProps {
  onExportExcel: () => void;
  onOpenChange?: (open: boolean) => void;
}

export function ExportMenu({ onExportExcel, onOpenChange }: ExportMenuProps) {
  const { open, menuRef, toggle, close } = useDropdownMenu({ onOpenChange });

  return (
    <div className="relative" ref={menuRef}>
      <Button
        aria-expanded={open}
        aria-haspopup="menu"
        aria-label="Export"
        className="size-8"
        size="icon"
        title="Export"
        variant="outline"
        onClick={toggle}
      >
        <UploadIcon className="size-3.5" />
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
        </DropdownPanel>
      ) : null}
    </div>
  );
}

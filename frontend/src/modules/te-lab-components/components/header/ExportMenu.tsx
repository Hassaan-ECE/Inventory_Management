import { FileSpreadsheetIcon, ListChecksIcon, UploadIcon } from "lucide-react";

import {
  DropdownItem,
  DropdownPanel,
} from "@/shared/components/ui/DropdownMenu";
import { useDropdownMenu } from "@/shared/hooks/useDropdownMenu";
import { Button } from "@/shared/components/ui/button";

interface ExportMenuProps {
  onExportExcel: () => void;
  onSelectForOrder: () => void;
  onOpenChange?: (open: boolean) => void;
}

export function ExportMenu({ onExportExcel, onSelectForOrder, onOpenChange }: ExportMenuProps) {
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
        <DropdownPanel align="right" className="w-64" maxHeightClassName="max-h-none" title="Export">
          <DropdownItem
            onClick={() => {
              close();
              onExportExcel();
            }}
          >
            <FileSpreadsheetIcon className="size-4" />
            Full Catalog Excel
          </DropdownItem>
          <DropdownItem
            onClick={() => {
              close();
              onSelectForOrder();
            }}
          >
            <ListChecksIcon className="size-4" />
            Select Components for Order
          </DropdownItem>
        </DropdownPanel>
      ) : null}
    </div>
  );
}

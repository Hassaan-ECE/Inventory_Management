export interface AttributeTemplate {
  key: string;
  label: string;
  unit?: string;
}

export interface CategoryTemplate {
  category: string;
  subcategories: string[];
  attributes: AttributeTemplate[];
}

export const CATEGORY_TEMPLATES: CategoryTemplate[] = [
  {
    category: "Passive",
    subcategories: ["Resistor", "Capacitor", "Inductor", "Ferrite", "Potentiometer"],
    attributes: [
      { key: "value", label: "Electrical value" },
      { key: "tolerance", label: "Tolerance", unit: "%" },
      { key: "powerRating", label: "Power rating", unit: "W" },
      { key: "ratedVoltage", label: "Rated voltage", unit: "V" },
      { key: "dielectric", label: "Dielectric" },
      { key: "polarity", label: "Polarity" },
      { key: "temperatureCoefficient", label: "Temperature coefficient" },
    ],
  },
  {
    category: "Semiconductor",
    subcategories: ["Diode", "BJT", "MOSFET", "JFET", "LED", "Rectifier"],
    attributes: [
      { key: "deviceType", label: "Device type" },
      { key: "voltageRating", label: "Voltage rating", unit: "V" },
      { key: "currentRating", label: "Current rating", unit: "A" },
      { key: "powerRating", label: "Power rating", unit: "W" },
      { key: "gain", label: "Gain" },
      { key: "wavelength", label: "Wavelength", unit: "nm" },
    ],
  },
  {
    category: "Integrated Circuit",
    subcategories: ["Op-amp", "Logic", "Regulator", "MCU", "Interface", "Driver"],
    attributes: [
      { key: "function", label: "Function" },
      { key: "supplyVoltage", label: "Supply voltage", unit: "V" },
      { key: "channelCount", label: "Channel count" },
      { key: "outputCurrent", label: "Output current", unit: "A" },
      { key: "protocol", label: "Protocol" },
      { key: "temperatureRange", label: "Temperature range" },
    ],
  },
  {
    category: "Connector",
    subcategories: ["Header", "Terminal Block", "Housing", "Contact", "Socket"],
    attributes: [
      { key: "positions", label: "Positions" },
      { key: "pitch", label: "Pitch", unit: "mm" },
      { key: "rows", label: "Rows" },
      { key: "gender", label: "Gender" },
      { key: "currentRating", label: "Current rating", unit: "A" },
      { key: "wireGauge", label: "Wire gauge", unit: "AWG" },
    ],
  },
  {
    category: "Electromechanical",
    subcategories: ["Relay", "Switch", "Fan", "Motor", "Buzzer"],
    attributes: [
      { key: "coilVoltage", label: "Coil voltage", unit: "V" },
      { key: "contactRating", label: "Contact rating" },
      { key: "actuator", label: "Actuator" },
      { key: "dimensions", label: "Dimensions" },
    ],
  },
  {
    category: "Module / Board",
    subcategories: ["Development Board", "Sensor Module", "Converter"],
    attributes: [
      { key: "function", label: "Function" },
      { key: "interface", label: "Interface" },
      { key: "supplyVoltage", label: "Supply voltage", unit: "V" },
      { key: "firmwareReference", label: "Firmware / reference" },
    ],
  },
  {
    category: "Cable / Wire",
    subcategories: ["Hook-up Wire", "Ribbon Cable", "Harness"],
    attributes: [
      { key: "conductorCount", label: "Conductor count" },
      { key: "wireGauge", label: "Wire gauge", unit: "AWG" },
      { key: "length", label: "Length" },
      { key: "color", label: "Color" },
      { key: "insulation", label: "Insulation" },
    ],
  },
  {
    category: "Hardware / Mechanical",
    subcategories: ["Standoff", "Screw", "Heat Sink", "Enclosure"],
    attributes: [
      { key: "thread", label: "Thread" },
      { key: "dimensions", label: "Dimensions" },
      { key: "material", label: "Material" },
    ],
  },
  {
    category: "Other",
    subcategories: [],
    attributes: [],
  },
];

export function categoryTemplate(category: string): CategoryTemplate | undefined {
  return CATEGORY_TEMPLATES.find((template) => template.category === category);
}

export function humanizeAttributeKey(key: string): string {
  return key
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .replace(/[_-]+/g, " ")
    .replace(/^./, (character) => character.toUpperCase());
}

export function normalizeAttributeKey(value: string): string {
  const words = value.trim().split(/[^a-zA-Z0-9]+/).filter(Boolean);
  if (words.length === 0) {
    return "";
  }
  return words
    .map((word, index) => {
      const lower = word.toLowerCase();
      return index === 0 ? lower : `${lower.charAt(0).toUpperCase()}${lower.slice(1)}`;
    })
    .join("");
}

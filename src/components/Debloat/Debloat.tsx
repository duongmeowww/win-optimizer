import { Card, Empty } from "antd";
import { useTranslation } from "react-i18next";

export default function Debloat() {
  const { t } = useTranslation();
  return (
    <Card title={t("nav.debloat")}>
      <Empty description="Module coming soon" />
    </Card>
  );
}

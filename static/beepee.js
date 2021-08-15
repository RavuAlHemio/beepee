"use strict";
;
var BeePee;
(function (BeePee) {
    BeePee.tsToSystolic = [];
    BeePee.tsToDiastolic = [];
    BeePee.tsToPulse = [];
    BeePee.tsToSpo2 = [];
    BeePee.todToSystolic = [];
    BeePee.todToDiastolic = [];
    BeePee.todToPulse = [];
    BeePee.todToSpo2 = [];
    function comparePoints(p1, p2) {
        let diff = p1.x - p2.x;
        if (diff != 0.0) {
            return diff;
        }
        diff = p1.y - p2.y;
        return diff;
    }
    function sortedClonePoints(points) {
        let newPoints = points.map(p => ({ x: p.x, y: p.y }));
        newPoints.sort(comparePoints);
        return newPoints;
    }
    function createCharts() {
        let tsChartCanvas = document.getElementById('ts-chart-canvas');
        let tsChartContext = tsChartCanvas.getContext("2d");
        if (tsChartContext === null) {
            console.error("failed to create time-of-day canvas context");
            return;
        }
        new Chart(tsChartContext, {
            type: "scatter",
            data: {
                datasets: [
                    {
                        label: "systolic",
                        data: sortedClonePoints(BeePee.tsToSystolic),
                        parsing: false,
                        borderColor: "#f00",
                    },
                    {
                        label: "diastolic",
                        data: sortedClonePoints(BeePee.tsToDiastolic),
                        parsing: false,
                        borderColor: "#00f",
                    },
                    {
                        label: "pulse",
                        data: sortedClonePoints(BeePee.tsToPulse),
                        parsing: false,
                        borderColor: "#0f0",
                    },
                    {
                        label: "spo2",
                        data: sortedClonePoints(BeePee.tsToSpo2),
                        parsing: false,
                        borderColor: "#fc0",
                    },
                ],
            },
            options: {
                animations: false,
                scales: {
                    xAxis: {
                        type: "time",
                    },
                },
            },
        });
        let todChartCanvas = document.getElementById('tod-chart-canvas');
        let todChartContext = todChartCanvas.getContext("2d");
        if (todChartContext === null) {
            console.error("failed to create time-of-day canvas context");
            return;
        }
        new Chart(todChartContext, {
            type: "scatter",
            data: {
                datasets: [
                    {
                        label: "systolic",
                        data: sortedClonePoints(BeePee.todToSystolic),
                        parsing: false,
                        borderColor: "#f00",
                    },
                    {
                        label: "diastolic",
                        data: sortedClonePoints(BeePee.todToDiastolic),
                        parsing: false,
                        borderColor: "#00f",
                    },
                    {
                        label: "pulse",
                        data: sortedClonePoints(BeePee.todToPulse),
                        parsing: false,
                        borderColor: "#0f0",
                    },
                    {
                        label: "spo2",
                        data: sortedClonePoints(BeePee.todToSpo2),
                        parsing: false,
                        borderColor: "#fc0",
                    },
                ],
            },
            options: {
                animations: false,
                scales: {
                    xAxis: {
                        type: "time",
                    },
                },
            },
        });
    }
    function setUp() {
        document.addEventListener("DOMContentLoaded", createCharts);
    }
    BeePee.setUp = setUp;
})(BeePee || (BeePee = {}));
//# sourceMappingURL=beepee.js.map
"use strict";

declare class Chart { constructor(context: CanvasRenderingContext2D, settings: any); };

module BeePee {
    interface Point {
        x: number;
        y: number;
    }

    export let tsToSystolic: Point[] = [];
    export let tsToDiastolic: Point[] = [];
    export let tsToPulse: Point[] = [];
    export let tsToSpo2: Point[] = [];

    export let todToSystolic: Point[] = [];
    export let todToDiastolic: Point[] = [];
    export let todToPulse: Point[] = [];
    export let todToSpo2: Point[] = [];

    function comparePoints(p1: Point, p2: Point): number {
        let diff = p1.x - p2.x;
        if (diff != 0.0) {
            return diff;
        }
        diff = p1.y - p2.y;
        return diff;
    }

    function sortedClonePoints(points: Point[]): Point[] {
        let newPoints = points.map(p => <Point>{x: p.x, y: p.y});
        newPoints.sort(comparePoints);
        return newPoints;
    }

    function createCharts() {
        let tsChartCanvas = <HTMLCanvasElement>document.getElementById('ts-chart-canvas');
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
                        data: sortedClonePoints(tsToSystolic),
                        parsing: false,
                        borderColor: "#f00",
                    },
                    {
                        label: "diastolic",
                        data: sortedClonePoints(tsToDiastolic),
                        parsing: false,
                        borderColor: "#00f",
                    },
                    {
                        label: "pulse",
                        data: sortedClonePoints(tsToPulse),
                        parsing: false,
                        borderColor: "#0f0",
                    },
                    {
                        label: "spo2",
                        data: sortedClonePoints(tsToSpo2),
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

        let todChartCanvas = <HTMLCanvasElement>document.getElementById('tod-chart-canvas');
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
                        data: sortedClonePoints(todToSystolic),
                        parsing: false,
                        borderColor: "#f00",
                    },
                    {
                        label: "diastolic",
                        data: sortedClonePoints(todToDiastolic),
                        parsing: false,
                        borderColor: "#00f",
                    },
                    {
                        label: "pulse",
                        data: sortedClonePoints(todToPulse),
                        parsing: false,
                        borderColor: "#0f0",
                    },
                    {
                        label: "spo2",
                        data: sortedClonePoints(todToSpo2),
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

    export function setUp() {
        document.addEventListener("DOMContentLoaded", createCharts);
    }
}
